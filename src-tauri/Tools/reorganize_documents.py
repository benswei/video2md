"""将 TXT / Markdown / PDF 按提示词重整为 Markdown，支持多个文件合并为系列课程。"""
import argparse, json, os, pathlib, sys

from transcribe import optimize_to_markdown

def read_document(path: str) -> str:
    suffix = pathlib.Path(path).suffix.lower()
    if suffix in (".txt", ".md", ".markdown", ".srt"):
        return pathlib.Path(path).read_text(encoding="utf-8", errors="replace")
    if suffix == ".pdf":
        try:
            from pypdf import PdfReader
        except ImportError as exc:
            raise RuntimeError("整理 PDF 需要 pypdf，请运行 pip install pypdf") from exc
        return "\n\n".join(page.extract_text() or "" for page in PdfReader(path).pages)
    raise RuntimeError(f"不支持的文件类型：{suffix}")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--inputs", required=True, help="JSON 文件路径数组")
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--provider", required=True, choices=["gemini", "custom"])
    parser.add_argument("--api-key")
    parser.add_argument("--api-url")
    parser.add_argument("--model-name")
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--series", action="store_true")
    args = parser.parse_args()
    paths = json.loads(args.inputs)
    if not paths: raise RuntimeError("没有输入文件")
    os.makedirs(args.output_dir, exist_ok=True)
    if args.series:
        combined = "\n\n".join(f"# 原始资料：{pathlib.Path(p).stem}\n\n{read_document(p)}" for p in paths)
        prompt = args.prompt + "\n\n这是系列课程/多份零散资料。请先重新设计统一大纲，再按新的章节体系合并重写；不要逐文件机械对应，输出一篇完整的干货文档。"
        text = optimize_to_markdown(combined, args.provider, args.api_url, args.api_key, args.model_name, prompt)
        output = pathlib.Path(args.output_dir) / "系列课程_重整.md"
        output.write_text(text, encoding="utf-8")
        print(f"OUTPUT:{output}", flush=True)
    else:
        for p in paths:
            text = optimize_to_markdown(read_document(p), args.provider, args.api_url, args.api_key, args.model_name, args.prompt)
            output = pathlib.Path(args.output_dir) / f"{pathlib.Path(p).stem}_重整.md"
            output.write_text(text, encoding="utf-8")
            print(f"OUTPUT:{output}", flush=True)

if __name__ == "__main__":
    main()
