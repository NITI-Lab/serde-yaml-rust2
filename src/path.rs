use std::fmt;

#[derive(Debug, Clone)]
pub(crate) enum PathSegment {
    Index(usize),
    Key(String),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Path {
    segments: Vec<PathSegment>,
}

impl Path {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push_index(&mut self, index: usize) {
        self.segments.push(PathSegment::Index(index));
    }

    pub(crate) fn push_key(&mut self, key: String) {
        self.segments.push(PathSegment::Key(key));
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.segments.is_empty() {
            return write!(f, ".");
        }
        for segment in &self.segments {
            match segment {
                PathSegment::Index(i) => write!(f, "[{i}]")?,
                PathSegment::Key(k) => write!(f, ".{k}")?,
            }
        }
        Ok(())
    }
}
