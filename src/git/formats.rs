/// Common git log and rev formats used across Twigdrop.
pub const FORMAT_COMMIT_WITH_METADATA: &str = "--format=%h|%s|%cr|%an";
pub const FORMAT_SHORT_HASH_MESSAGE: &str = "--format=%h %s";
pub const FORMAT_HASH_PIPE_MESSAGE: &str = "--format=%h|%s";
pub const FORMAT_ISO_DATE: &str = "--format=%cI";
pub const FORMAT_STASH_LIST: &str = "--format=%gd|%s";
pub const FORMAT_REFNAME_SHORT: &str = "--format=%(refname:short)";
pub const FORMAT_REFNAME_MERGED: &str = "--format=%(refname:short)";
