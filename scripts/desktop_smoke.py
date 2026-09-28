"""Drive the real desktop window through WebKit's WebDriver.

Run by scripts/desktop-smoke.sh, which provides the isolated home, the
Broadway display and the WebDriver server this expects; see there.

What it does, in the real application with the real engine behind it:

1. Sets a device up from nothing through the window -- folder, storage, the 24
   words and three of them confirmed.
2. Opens every tab.
3. Shows a pairing code, and has a second device (the command line, enrolled
   with the same words) join with it.
4. Sends that device a file.
5. Removes that device, from its details, answering the question.

It fails if any command the window calls returns an error, whichever step
called it, or if a step does not reach the state it should. It checks that
the window works, not how it looks: the Broadway display reports a nonsense
geometry, so screenshots from it are not worth reading.
"""

import base64
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request

DRIVER = os.environ["SMOKE_DRIVER"]
APP = os.environ["SMOKE_APP"]
QURB = os.environ["SMOKE_QURB"]
HOME = os.environ["HOME"]


def call(method, path, body=None):
    request = urllib.request.Request(
        DRIVER + path,
        method=method,
        data=None if body is None else json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as answer:
            return json.load(answer)["value"]
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"{method} {path}: {e.read().decode()[:400]}") from None


class Window:
    def __init__(self):
        caps = {"capabilities": {"alwaysMatch": {
            "webkitgtk:browserOptions": {"binary": APP, "args": []}}}}
        self.id = call("POST", "/session", caps)["sessionId"]

    def js(self, script, *args):
        return call("POST", f"/session/{self.id}/execute/sync",
                    {"script": script, "args": list(args)})

    def click(self, element_id):
        self.js("document.getElementById(arguments[0]).click()", element_id)

    def until(self, what, script, *args, timeout=20):
        """Wait for a script to return something truthy, and return it."""
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            value = self.js(script, *args)
            if value:
                return value
            time.sleep(0.2)
        raise AssertionError(f"never happened: {what}")

    def screenshot(self, path):
        with open(path, "wb") as f:
            f.write(base64.b64decode(call("GET", f"/session/{self.id}/screenshot")))

    def close(self):
        try:
            call("DELETE", f"/session/{self.id}")
        except Exception:
            pass


def visible(step):
    return (f"return !document.querySelector('[data-step={step}]')"
            f".classList.contains('hidden') && document.querySelector('[data-step={step}]')"
            f".classList.contains('on')")


def set_up(window):
    window.until("the setting-up screens", "return !document.getElementById('setup').classList.contains('hidden')")
    window.click("choose-new")
    window.js("""const f = document.getElementById('folder-path');
                 f.value = arguments[0]; f.dispatchEvent(new Event('input'));""", f"{HOME}/qurb")
    window.until("the folder accepted", "return !document.getElementById('folder-next').disabled")
    window.click("folder-next")

    window.until("the storage presets", "return document.querySelectorAll('#allowances button:not([disabled])').length")
    window.js("document.querySelector('#allowances button:not([disabled])').click()")
    window.until("a size chosen", "return !document.getElementById('storage-next').disabled")
    window.click("storage-next")

    words = window.until("the 24 words",
        "const w = [...document.querySelectorAll('#words li')].map(l => l.textContent); return w.length === 24 && w")
    window.click("phrase-next")
    window.js("""for (const input of document.querySelectorAll('#asks input'))
                   input.value = arguments[0][Number(input.dataset.position) - 1];""", words)
    window.click("verify-next")
    window.until("the ready screen", visible("ready"))
    window.click("ready-next")
    window.until("the main window", "return document.getElementById('setup').classList.contains('hidden')")
    return words


def every_tab(window):
    tabs = window.js("return [...document.querySelectorAll('#tabs button')].map(b => b.dataset.screen)")
    for tab in tabs:
        window.js("document.querySelector(`#tabs [data-screen=${arguments[0]}]`).click()", tab)
        window.until(f"the {tab} tab", "return !document.getElementById(arguments[0]).classList.contains('hidden')", tab)
        time.sleep(0.5)
    return tabs


def pair(window, words):
    other = f"{HOME}/other"
    subprocess.run([QURB, "enrol", other, " ".join(words)], check=True, capture_output=True)

    window.js("document.querySelector('#tabs [data-screen=devices]').click()")
    window.click("pair-show")
    code = window.until("a code on the screen",
        "const t = document.getElementById('pair-code').textContent; return t && t.split(' ').pop()")
    qr = window.js("return document.getElementById('qr').innerHTML.length")
    assert qr > 1000, f"no QR drawn ({qr} characters)"

    joined = subprocess.run([QURB, "join", other, code], capture_output=True, text=True, timeout=60)
    assert joined.returncode == 0, f"qurb join failed: {joined.stdout}{joined.stderr}"
    named = window.until("the window saying paired",
        "return !document.getElementById('pair-done').classList.contains('hidden') && document.getElementById('pair-with').textContent")
    window.click("pair-finish")
    return other, named


def send(window, device):
    path = f"{HOME}/to-send.txt"
    with open(path, "w") as f:
        f.write("sent by the desktop smoke test\n")
    # Called directly: the page chooses files through the system's file
    # dialog, which WebDriver cannot drive. execute/sync cannot wait on a
    # promise, so it is started and its result polled for.
    window.js("""window.__sent = null;
        window.__TAURI__.core.invoke('send_files', {paths: [arguments[0]], to: arguments[1]})
          .then(r => { window.__sent = r }, e => { window.__sent = {error: String(e)} });""", path, device)
    report = window.until("the send to finish", "return window.__sent")
    assert "error" not in report, f"send failed: {report['error']}"
    assert report["sent"] == 1, f"sent {report}"

    window.js("document.querySelector('#tabs [data-screen=transfers]').click()")
    window.until("the send listed under Transfers",
        "return document.getElementById('transfers').textContent.includes('to-send.txt')")


def remove(window, device):
    """Remove the paired device through its details, and see it gone.

    The file sent to it was never collected -- the other device is a command
    line that never ran -- so the question must say a send will be cancelled.
    """
    window.js("document.querySelector('#tabs [data-screen=devices]').click()")
    window.until("the device listed", "return document.querySelector('#device-list details')")
    window.js("""const d = document.querySelector('#device-list details'); d.open = true;
                 [...d.querySelectorAll('button')].find(b => b.textContent.startsWith('Remove')).click();""")
    said = window.until("the question", "const c = document.querySelector('.confirm'); return c && c.textContent")
    assert f"Remove {device}?" in said, said
    assert "1 file waiting for it to collect will be cancelled" in said, said
    window.js("[...document.querySelectorAll('.confirm button')].find(b => b.textContent === 'Remove device').click()")
    window.until("the device gone",
        "return document.getElementById('device-list').textContent.includes('no paired devices yet')")
    window.js("document.querySelector('#tabs [data-screen=transfers]').click()")
    window.until("the send no longer waiting",
        "return !document.getElementById('transfers').textContent.includes('waiting for')" )


def main():
    window = Window()
    failures = []
    ok = False
    try:
        time.sleep(2)
        words = set_up(window)
        print("set up through the window")

        print("opened", ", ".join(every_tab(window)))

        other, named = pair(window, words)
        print(f"paired by code with {named}")

        device = named.split(" (")[0]
        send(window, device)
        print(f"sent a file to {device}")
        remove(window, device)
        print(f"removed {device}, and its waiting send with it")
        ok = True
    except Exception:
        try:
            window.screenshot(os.path.join(HOME, "failed.png"))
        except Exception:
            pass
        raise
    finally:
        try:
            failures = window.js("return window.qurbFailures || []")
        except Exception:
            pass
        window.close()
        # Printed on the way out of a failed step too: which command failed,
        # and with what, is usually the reason the step did not finish.
        if failures:
            print("commands that failed:", *failures, sep="\n  ")

    if failures or not ok:
        sys.exit(1)
    print("no command failed")


if __name__ == "__main__":
    main()
