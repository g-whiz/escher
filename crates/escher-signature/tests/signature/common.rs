use escher_signature::{Error, ErrorKind, Signature};

pub(crate) fn parse(text: &str) -> Result<Signature, Error> {
    text.parse()
}

pub(crate) fn signature(text: &str) -> Signature {
    match parse(text) {
        Ok(signature) => signature,
        Err(error) => panic!("{text:?} is rejected: {error}"),
    }
}

pub(crate) fn canonical(text: &str) -> String {
    match parse(text) {
        Ok(signature) => signature.to_string(),
        Err(error) => panic!("{text:?} is rejected: {error}"),
    }
}

pub(crate) fn kind_and_span(text: &str) -> (ErrorKind, core::ops::Range<usize>) {
    match parse(text) {
        Ok(signature) => panic!("{text:?} is accepted as {signature}"),
        Err(error) => (error.kind(), error.span()),
    }
}

pub(crate) fn chi(text: &str) -> String {
    match parse(text) {
        Ok(signature) => signature.euler_characteristic().to_string(),
        Err(error) => panic!("{text:?} is rejected: {error}"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Value {
    Finite(u32),
    Infinite,
}

impl Value {
    pub(crate) fn text(self) -> String {
        match self {
            Self::Finite(value) if value < 10 => value.to_string(),
            Self::Finite(value) => format!("({value})"),
            Self::Infinite => "(inf)".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Model {
    pub(crate) handles: u32,
    pub(crate) crosscaps: u32,
    pub(crate) cones: Vec<Value>,
    pub(crate) boundaries: Vec<Vec<Value>>,
}

impl Model {
    pub(crate) fn of(signature: &Signature) -> Self {
        let value =
            |order: &escher_signature::Order| order.finite().map_or(Value::Infinite, Value::Finite);
        Self {
            handles: signature.handles(),
            crosscaps: signature.crosscaps(),
            cones: signature.cone_orders().iter().map(value).collect(),
            boundaries: signature
                .boundaries()
                .iter()
                .map(|boundary| boundary.corner_orders().iter().map(value).collect())
                .collect(),
        }
    }

    pub(crate) fn tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        for _ in 0..self.handles {
            tokens.push("o".to_owned());
        }
        tokens.extend(self.cones.iter().map(|value| value.text()));
        for corners in &self.boundaries {
            tokens.push("*".to_owned());
            tokens.extend(corners.iter().map(|value| value.text()));
        }
        for _ in 0..self.crosscaps {
            tokens.push("x".to_owned());
        }
        tokens
    }

    pub(crate) fn text(&self) -> String {
        let text = self.tokens().concat();
        if text.is_empty() {
            "1".to_owned()
        } else {
            text
        }
    }

    pub(crate) fn moves(&self) -> Vec<Self> {
        let mut moves = Vec::new();
        for index in 1..self.cones.len() {
            let mut next = self.clone();
            next.cones.swap(index - 1, index);
            moves.push(next);
        }
        for index in 1..self.boundaries.len() {
            let mut next = self.clone();
            next.boundaries.swap(index - 1, index);
            moves.push(next);
        }
        for index in 0..self.boundaries.len() {
            if self.boundaries[index].len() > 1 {
                let mut next = self.clone();
                next.boundaries[index].rotate_left(1);
                moves.push(next);
            }
        }
        if self.boundaries.iter().any(|corners| corners.len() > 2) {
            let mut next = self.clone();
            next.boundaries
                .iter_mut()
                .for_each(|corners| corners.reverse());
            moves.push(next);
        }
        if self.crosscaps > 0 {
            for index in 0..self.boundaries.len() {
                if self.boundaries[index].len() > 2 {
                    let mut next = self.clone();
                    next.boundaries[index].reverse();
                    moves.push(next);
                }
            }
        }
        if self.handles > 0 && self.crosscaps > 0 {
            let mut next = self.clone();
            next.handles -= 1;
            next.crosscaps += 2;
            moves.push(next);
        }
        if self.crosscaps > 2 {
            let mut next = self.clone();
            next.handles += 1;
            next.crosscaps -= 2;
            moves.push(next);
        }
        moves
    }
}

pub(crate) fn assert_same_result(first: &str, second: &str) {
    match (parse(first), parse(second)) {
        (Ok(left), Ok(right)) => {
            assert_eq!(left, right, "{first:?} and {second:?}");
            assert_eq!(left.to_string(), right.to_string());
            assert_eq!(left.euler_characteristic(), right.euler_characteristic());
        }
        (Err(left), Err(right)) => {
            assert_eq!(left.kind(), right.kind(), "{first:?} and {second:?}");
        }
        (left, right) => panic!("{first:?} gives {left:?} and {second:?} gives {right:?}"),
    }
}
