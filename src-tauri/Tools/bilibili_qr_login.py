import argparse
import json
import sys
import requests

HEADERS = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120 Safari/537.36", "Referer": "https://www.bilibili.com/"}

def output(value):
    print(json.dumps(value, ensure_ascii=False), flush=True)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["start", "poll"])
    parser.add_argument("--key", default="")
    args = parser.parse_args()
    if args.action == "start":
        response = requests.get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate", headers=HEADERS, timeout=12)
        data = response.json().get("data", {})
        if not data.get("qrcode_key") or not data.get("url"):
            raise RuntimeError("B站未返回二维码")
        output({"status": "pending", "key": data["qrcode_key"], "url": data["url"]})
        return
    response = requests.get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll", params={"qrcode_key": args.key}, headers=HEADERS, timeout=12)
    payload = response.json().get("data", {})
    code = payload.get("code")
    status = {86101: "pending", 86090: "scanned", 86038: "expired", 0: "confirmed"}.get(code, "error")
    result = {"status": status, "message": payload.get("message", "")}
    if status == "confirmed":
        result["cookies"] = requests.utils.dict_from_cookiejar(response.cookies)
    output(result)

if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        output({"status": "error", "message": str(exc)})
        sys.exit(1)
