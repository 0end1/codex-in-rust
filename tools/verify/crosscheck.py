#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""跨章符号一致性审计：
1) 提取全书 rust 块中定义的顶层标识符(struct/enum/trait/type/fn/const) 及其出现章位；
2) 统计全部 PascalCase 标识符引用频次，扣除【定义点】与【std/第三方白名单】后，
   输出“被引用却全书无定义”的候选符号（潜在笔误/不存在的 API）。
用法: python3 crosscheck.py <md>
"""
import re, sys, json
from collections import Counter, defaultdict

MD = sys.argv[1]
lines = open(MD, encoding="utf-8").read().split("\n")
fence = re.compile(r"^```([\w,.-]*)\s*$")
ch_re = re.compile(r"^# 第\s*(\d+)\s*章")

blocks = []   # (ch, buf)
ch = "pre"
i = 0
while i < len(lines):
    m = fence.match(lines[i])
    if m and m.group(1) in ("rust", "rust,ignore"):
        i += 1
        buf = []
        while i < len(lines) and not fence.match(lines[i]):
            cm = ch_re.match(lines[i])
            if cm: ch = int(cm.group(1))
            buf.append(lines[i]); i += 1
        blocks.append((ch, buf))
    else:
        cm = ch_re.match(lines[i])
        if cm: ch = int(cm.group(1))
        i += 1

# ---- 定义提取 ----
defs = defaultdict(list)          # name -> [(ch, kind, line_text)]
fn_defs = defaultdict(list)
enum_variants = defaultdict(list) # enum name -> variants
fields = defaultdict(list)
for ch, buf in blocks:
    text = "\n".join(buf)
    # 去掉行注释与字符串，减少噪音（粗处理）
    code = re.sub(r"//.*", "", text)
    code = re.sub(r'"(?:[^"\\]|\\.)*"', '""', code)
    code = re.sub(r"r#\".*?\"#", "", code)
    for m in re.finditer(r"(?m)^\s*pub(?:\([^)]*\))?\s+(?:struct|enum|trait|type|fn|const)\s+([A-Za-z_]\w*)", code):
        defs[m.group(1)].append((ch, "pub-item", m.group(0).strip()[:60]))
    for m in re.finditer(r"(?m)^\s*(?:struct|enum|trait|type|fn|const|async\s+fn)\s+([A-Za-z_]\w*)", code):
        defs[m.group(1)].append((ch, "item", m.group(0).strip()[:60]))
    # struct/enum 定义块：用括号配对取体，分别抓字段与变体
    for dm in re.finditer(r"(struct|enum|trait|type)\s+([A-Za-z_]\w*)", code):
        kind, name = dm.group(1), dm.group(2)
        p = dm.end()
        # 跳到 {（可能带 where / 泛型）
        open_idx = code.find("{", p)
        if open_idx < 0:
            continue
        depth = 0
        j = open_idx
        while j < len(code):
            if code[j] == "{":
                depth += 1
            elif code[j] == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        body = code[open_idx + 1:j]
        if kind == "enum":
            for vm in re.finditer(r"(?m)^\s*([A-Z][A-Za-z0-9_]*)\s*(?:[\(,\{]|\s*(?:=>|,|$))", body):
                if not re.search(r"^\s*(pub|where)", vm.group(0)):
                    enum_variants[name].append(vm.group(1))
        elif kind == "struct":
            for fm in re.finditer(r"(?m)^\s{4,}([a-z_]\w*)\s*:", body):
                fields[name].append(fm.group(1))

# ---- 引用统计 ----
PASCAL = re.compile(r"\b([A-Z][A-Za-z0-9_]*)\b")
uses = Counter()
claims = []   # (EnumName, Variant)
for ch, buf in blocks:
    code = re.sub(r"//.*", "", "\n".join(buf))
    code = re.sub(r'"(?:[^"\\]|\\.)*"', '""', code)
    for t in PASCAL.findall(code):
        uses[t] += 1
    for cm in re.finditer(r"([A-Z][A-Za-z0-9_]*)\s*::\s*([A-Z][A-Za-z0-9_]*)", code):
        claims.append((cm.group(1), cm.group(2)))

# 白名单（std / 常用第三方 / 属性名）
whitelist = set("""
String Vec Option Some None Result Ok Err Box Arc Rc Mutex RwLock RefCell Cell
HashMap BTreeMap HashSet BTreeSet VecDeque BinaryHeap LinkedList Duration Instant
Path PathBuf SystemTime Metadata Cursor Read Write Seek BufReader BufWriter
Default Debug Clone Copy PartialEq Eq PartialOrd Ord Hash Send Sync Future
IntoFuture BoxFuture Pin Pinned LocalSet JoinHandle Spawn Task Runtime Error
Into From TryFrom TryInto AsRef AsMut Deref DerefMut Iterator IntoIterator
Extend Collect iter Iterator Item DoubleEndedIterator ExactSizeIterator Fn FnMut FnOnce
ToOwned ToString ToSocketAddrs ToString Stdout Stderr Stdio Command Child ExitStatus ExitCode
PhantomData NonZero usize isize u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 bool char str
Unit Tuple Struct TupleStruct TupleVariant StructVariant NewtypeVariant
Serialize Deserialize Deserializer Serializer JsonValue Value Map Number
ToSql FromSql Row Stream StreamExt Sink Ext AsyncRead AsyncWrite AsyncBufRead
ReadBuf Poll Ready Context Waker Bytes BytesMut Buf BufMut
Attribute SystemProcess OpenOptions File Permissions
Uuid Serde Json Toml AsyncTrait Subprocess Builder Handler Error
DateTime Utc Local NaiveDate NaiveDateTime SerdeField UnboundedSender UnboundedReceiver
Sender Receiver SenderError ReceiveError RecvError TryRecvError TrySendError
CancellationToken Token Trio OptionExt ArcSwap RwLockReadGuard RwLockWriteGuard MutexGuard
LlmError LlmOutput HttpError NetworkError ClientError ServerError ProtocolError
""".split())
whitelist |= {t for t in uses if t in {"T", "E", "K", "V", "U", "R", "A", "B", "C", "D", "F", "S", "N", "X", "Y", "Z"}}
defined_names = set(defs.keys()) | {v for vs in enum_variants.values() for v in vs}
enum_names = set(enum_variants.keys())
known_variants = {v for vs in enum_variants.values() for v in vs}

# 声称的变体（X::Y，X 有 enum 定义）——与 enum 定义里实际出现的变体对照
claim_missing = set()
for t1, t2 in claims:
    if t1 in enum_names and t2 not in enum_variants[t1]:
        claim_missing.add((t1, t2))

# 由已知 enum 声明的变体不算 unknown
claimed_variants = {t2 for t1, t2 in claims if t1 in enum_names}

unknown = {t: n for t, n in uses.items()
           if n >= 2 and t not in whitelist
           and t not in defined_names and t not in claimed_variants
           and not t.isdigit()}

out = {
    "defs_count": len(defs),
    "top_unknown": sorted(unknown.items(), key=lambda kv: -kv[1])[:60],
    "claim_missing_variants": sorted(claim_missing),
    "enums": {k: list(dict.fromkeys(v)) for k, v in enum_variants.items()},
}
# 顶层 defs 按名称输出章位（探测重复定义）
def_locs = {}
for k, v in defs.items():
    def_locs[k] = [(ch, kind) for ch, kind, _ in v]
out["def_locations"] = {k: v for k, v in sorted(def_locs.items()) if len(v) <= 4}

json.dump(out, open("/Users/wangzhiyong/Desktop/《用Rust造一个Codex》修订项目/_verify_out/crosscheck.json", "w"), ensure_ascii=False, indent=1)
print("defs:", len(defs), " enums:", len(enum_variants))
print("\n== top unknown (used>=2, no def, not std) ==")
for t, n in out["top_unknown"]:
    print("  %5d  %s" % (n, t))
