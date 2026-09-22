use serde::{Deserialize, Serialize};
use std::fmt;

/// 每一类损坏都有独立的错误变体，便于界面分开报告：
/// 位图越界、stage 冲突、扩展截断、校验和错误、递归稀疏目录等。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail")]
pub enum IndexError {
    BadMagic,
    UnsupportedVersion(u32),
    Truncated { what: String, offset: usize },
    BadVarint { offset: usize },
    BadEntryFlags { offset: usize, reason: String },
    ChecksumMismatch { expected: String, actual: String },
    TruncatedExtension { signature: String, offset: usize, declared: u32, remaining: usize },
    ExtensionMalformed { signature: String, reason: String },
    BitmapOutOfRange { kind: String, bit: u32, base_len: usize },
    StageConflict { path: Vec<u8>, stage: u8 },
    RecursiveSparseDirectory { path: Vec<u8> },
    MissingSharedIndex { hash: String },
    MissingLinkExtension { file: String },
}

impl IndexError {
    /// 稳定的机器可读类别名，供界面与测试断言。
    pub fn category(&self) -> &'static str {
        match self {
            IndexError::BadMagic => "bad-magic",
            IndexError::UnsupportedVersion(_) => "unsupported-version",
            IndexError::Truncated { .. } => "truncated",
            IndexError::BadVarint { .. } => "bad-varint",
            IndexError::BadEntryFlags { .. } => "bad-entry-flags",
            IndexError::ChecksumMismatch { .. } => "checksum-mismatch",
            IndexError::TruncatedExtension { .. } => "truncated-extension",
            IndexError::ExtensionMalformed { .. } => "extension-malformed",
            IndexError::BitmapOutOfRange { .. } => "bitmap-out-of-range",
            IndexError::StageConflict { .. } => "stage-conflict",
            IndexError::RecursiveSparseDirectory { .. } => "recursive-sparse-directory",
            IndexError::MissingSharedIndex { .. } => "missing-shared-index",
            IndexError::MissingLinkExtension { .. } => "missing-link-extension",
        }
    }
}

fn show_path(path: &[u8]) -> String {
    path.iter()
        .map(|b| {
            if (0x20..0x7f).contains(b) {
                (*b as char).to_string()
            } else {
                format!("\\x{b:02x}")
            }
        })
        .collect()
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexError::BadMagic => write!(f, "文件头不是 DIRC 魔数"),
            IndexError::UnsupportedVersion(v) => write!(f, "不支持的 index 版本 {v}（仅支持 2/3/4）"),
            IndexError::Truncated { what, offset } => {
                write!(f, "{what} 在偏移 {offset} 处被截断")
            }
            IndexError::BadVarint { offset } => write!(f, "偏移 {offset} 处的 v4 前缀长度 varint 非法"),
            IndexError::BadEntryFlags { offset, reason } => {
                write!(f, "偏移 {offset} 处的条目标志非法：{reason}")
            }
            IndexError::ChecksumMismatch { expected, actual } => {
                write!(f, "校验和错误：文件记录 {expected}，实际计算 {actual}")
            }
            IndexError::TruncatedExtension { signature, offset, declared, remaining } => write!(
                f,
                "扩展 {signature} 在偏移 {offset} 声明 {declared} 字节，但校验和前只剩 {remaining} 字节"
            ),
            IndexError::ExtensionMalformed { signature, reason } => {
                write!(f, "扩展 {signature} 内容畸形：{reason}")
            }
            IndexError::BitmapOutOfRange { kind, bit, base_len } => write!(
                f,
                "{kind} 位图第 {bit} 位超出基准索引的 {base_len} 个条目范围"
            ),
            IndexError::StageConflict { path, stage } => write!(
                f,
                "合并后路径 {} 的 stage {stage} 出现重复条目",
                show_path(path)
            ),
            IndexError::RecursiveSparseDirectory { path } => write!(
                f,
                "稀疏目录 {} 内嵌套了另一个稀疏目录",
                show_path(path)
            ),
            IndexError::MissingSharedIndex { hash } => write!(
                f,
                "缺少 shared index {hash}，只能展示主 index 可证实的部分"
            ),
            IndexError::MissingLinkExtension { file } => write!(
                f,
                "{file} 不含 link 扩展，无法作为 split-index 合并"
            ),
        }
    }
}

impl std::error::Error for IndexError {}
