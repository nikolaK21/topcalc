//! Small expression parser and 2D sampler.
//!
//! Syntax: `+ - * / ^ **`, parentheses, juxtaposition (`2x`, `2(x+1)`),
//! trig and inverse trig, hyperbolic functions, `sqrt`, `ln`, `log`, `log10`,
//! `log2`, `abs`, `floor`, `ceil`, `factorial`, finite `sum_{n=a}^{b}` and
//! `prod_{n=a}^{b}`, and `lim_{x->a}`. Trig is in radians. `log` is natural.
//! `^` and `**` are right-associative.

use std::f64::consts::{E, PI};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    Sqrt,
    Abs,
    Sin,
    Cos,
    Tan,
    Sec,
    Csc,
    Cot,
    Asin,
    Acos,
    Atan,
    Asec,
    Acsc,
    Acot,
    Sinh,
    Cosh,
    Tanh,
    Sech,
    Csch,
    Coth,
    Asinh,
    Acosh,
    Atanh,
    Ln,
    Log,
    Log2,
    Floor,
    Ceil,
    Round,
    Sign,
    Fact,
    Gamma,
    Digamma,
    Trigamma,
}

const FUNC_NAMES: &[(&str, Func)] = &[
    ("arcsinh", Func::Asinh),
    ("arccosh", Func::Acosh),
    ("arctanh", Func::Atanh),
    ("arcsec", Func::Asec),
    ("arccsc", Func::Acsc),
    ("arccot", Func::Acot),
    ("arcsin", Func::Asin),
    ("arccos", Func::Acos),
    ("arctan", Func::Atan),
    ("ceiling", Func::Ceil),
    ("factorial", Func::Fact),
    ("trigamma", Func::Trigamma),
    ("digamma", Func::Digamma),
    ("asinh", Func::Asinh),
    ("acosh", Func::Acosh),
    ("atanh", Func::Atanh),
    ("floor", Func::Floor),
    ("round", Func::Round),
    ("gamma", Func::Gamma),
    ("asec", Func::Asec),
    ("acsc", Func::Acsc),
    ("acot", Func::Acot),
    ("asin", Func::Asin),
    ("acos", Func::Acos),
    ("atan", Func::Atan),
    ("sinh", Func::Sinh),
    ("cosh", Func::Cosh),
    ("tanh", Func::Tanh),
    ("sech", Func::Sech),
    ("csch", Func::Csch),
    ("coth", Func::Coth),
    ("sqrt", Func::Sqrt),
    ("sign", Func::Sign),
    ("ceil", Func::Ceil),
    ("fact", Func::Fact),
    ("log2", Func::Log2),
    ("log10", Func::Log),
    ("abs", Func::Abs),
    ("sin", Func::Sin),
    ("cos", Func::Cos),
    ("tan", Func::Tan),
    ("sec", Func::Sec),
    ("csc", Func::Csc),
    ("cot", Func::Cot),
    ("sgn", Func::Sign),
    ("ln", Func::Ln),
    ("log", Func::Ln),
];

impl Func {
    fn from_name(name: &str) -> Option<Self> {
        FUNC_NAMES.iter().find(|(entry, _)| *entry == name).map(|(_, func)| *func)
    }

    fn source_name(self) -> &'static str {
        match self {
            Func::Sqrt => "sqrt",
            Func::Abs => "abs",
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Tan => "tan",
            Func::Sec => "sec",
            Func::Csc => "csc",
            Func::Cot => "cot",
            Func::Asin => "arcsin",
            Func::Acos => "arccos",
            Func::Atan => "arctan",
            Func::Asec => "arcsec",
            Func::Acsc => "arccsc",
            Func::Acot => "arccot",
            Func::Sinh => "sinh",
            Func::Cosh => "cosh",
            Func::Tanh => "tanh",
            Func::Sech => "sech",
            Func::Csch => "csch",
            Func::Coth => "coth",
            Func::Asinh => "arcsinh",
            Func::Acosh => "arccosh",
            Func::Atanh => "arctanh",
            Func::Ln => "ln",
            Func::Log => "log10",
            Func::Log2 => "log2",
            Func::Floor => "floor",
            Func::Ceil => "ceil",
            Func::Round => "round",
            Func::Sign => "sign",
            Func::Fact => "factorial",
            Func::Gamma => "gamma",
            Func::Digamma => "digamma",
            Func::Trigamma => "trigamma",
        }
    }

    fn apply(self, value: f64) -> f64 {
        match self {
            Func::Sqrt => value.sqrt(),
            Func::Abs => value.abs(),
            Func::Sin => value.sin(),
            Func::Cos => value.cos(),
            Func::Tan => value.tan(),
            Func::Sec => 1.0 / value.cos(),
            Func::Csc => 1.0 / value.sin(),
            Func::Cot => 1.0 / value.tan(),
            Func::Asin => value.asin(),
            Func::Acos => value.acos(),
            Func::Atan => value.atan(),
            Func::Asec => (1.0 / value).acos(),
            Func::Acsc => (1.0 / value).asin(),
            Func::Acot => PI / 2.0 - value.atan(),
            Func::Sinh => value.sinh(),
            Func::Cosh => value.cosh(),
            Func::Tanh => value.tanh(),
            Func::Sech => 1.0 / value.cosh(),
            Func::Csch => 1.0 / value.sinh(),
            Func::Coth => 1.0 / value.tanh(),
            Func::Asinh => (value + (value * value + 1.0).sqrt()).ln(),
            Func::Acosh => (value + (value * value - 1.0).sqrt()).ln(),
            Func::Atanh => 0.5 * ((1.0 + value) / (1.0 - value)).ln(),
            Func::Ln => value.ln(),
            Func::Log => value.log10(),
            Func::Log2 => value.log2(),
            Func::Floor => value.floor(),
            Func::Ceil => value.ceil(),
            Func::Round => value.round(),
            Func::Sign => {
                if value == 0.0 {
                    0.0
                } else {
                    value.signum()
                }
            },
            Func::Fact => factorial(value),
            Func::Gamma => gamma(value),
            Func::Digamma => digamma(value),
            Func::Trigamma => trigamma(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var,
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Call(Func, Box<Expr>),
    Index(String),
    Sum {
        var: String,
        lower: Box<Expr>,
        upper: Box<Expr>,
        body: Box<Expr>,
    },
    Prod {
        var: String,
        lower: Box<Expr>,
        upper: Box<Expr>,
        body: Box<Expr>,
    },
    Limit {
        var: String,
        target: Box<Expr>,
        body: Box<Expr>,
    },
}

impl Expr {
    pub fn uses_x(&self) -> bool {
        match self {
            Expr::Num(_) | Expr::Index(_) => false,
            Expr::Var => true,
            Expr::Neg(inner) | Expr::Call(_, inner) => inner.uses_x(),
            Expr::Add(a, b)
            | Expr::Sub(a, b)
            | Expr::Mul(a, b)
            | Expr::Div(a, b)
            | Expr::Pow(a, b) => a.uses_x() || b.uses_x(),
            Expr::Sum { lower, upper, body, .. } | Expr::Prod { lower, upper, body, .. } => {
                lower.uses_x() || upper.uses_x() || body.uses_x()
            }
            Expr::Limit { target, body, .. } => target.uses_x() || body.uses_x(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotSpec {
    pub label: String,
    pub expression: String,
    /// `function` is sampled as y = expression. `vertical` is the line x = expression.
    pub plot: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub plain: String,
    pub latex: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathPiece {
    pub label: String,
    /// Primary source. For `y = f(x)` this is `f(x)`, matching the sampler input.
    pub expression: String,
    pub uses_x: bool,
    pub value: Option<f64>,
    pub latex: String,
    pub result_label: String,
    pub result_plain: String,
    pub result_latex: String,
    pub note: Option<String>,
    pub plots: Vec<PlotSpec>,
    pub solutions: Vec<Solution>,
    pub markers: Vec<Marker>,
    /// Suggested window `(x_min, x_max, y_min, y_max)` when the solutions should be framed.
    pub frame: Option<(f64, f64, f64, f64)>,
}

pub fn parse_math_input(input: &str) -> Result<Vec<MathPiece>, String> {
    let mut raw = input.trim();
    if let Some(rest) = raw.strip_prefix('=') {
        raw = rest.trim();
    }
    if raw.is_empty() {
        return Err("Enter an expression".into());
    }
    let parts = split_pieces(raw);
    if parts.is_empty() {
        return Err("Enter an expression".into());
    }
    parts.iter().map(|part| parse_piece(part)).collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum YSide {
    Left,
    Right,
    None,
}

struct IntegralQuery {
    body: String,
    lower: Option<String>,
    upper: Option<String>,
}

fn parse_piece(part: &str) -> Result<MathPiece, String> {
    let label = part.trim().to_string();
    if label.is_empty() {
        return Err("Enter an expression".into());
    }
    let normalized = wolfram_normalize(&label);
    if let Some((lhs_src, rhs_src)) = split_equation(&normalized) {
        if lhs_src.is_empty() {
            return Err("Missing the left-hand side".into());
        }
        if rhs_src.is_empty() {
            return Err("Missing the right-hand side".into());
        }
        if lhs_src.eq_ignore_ascii_case("y") {
            if let Some(query) = parse_integral_query(&rhs_src)? {
                return dispatch_integral(&label, query, YSide::Left);
            }
            if let Some((order, body)) = parse_derivative_body(&rhs_src)? {
                return calculus_derivative(&label, &body, order, YSide::Left);
            }
            let rhs = parse_expression(&rhs_src)?;
            return Ok(function_piece(label, rhs_src, &rhs));
        }
        if rhs_src.eq_ignore_ascii_case("y") {
            if let Some(query) = parse_integral_query(&lhs_src)? {
                return dispatch_integral(&label, query, YSide::Right);
            }
            if let Some((order, body)) = parse_derivative_body(&lhs_src)? {
                return calculus_derivative(&label, &body, order, YSide::Right);
            }
            let lhs = parse_expression(&lhs_src)?;
            return Ok(function_piece(label, lhs_src, &lhs));
        }
        let lhs = parse_expression(&lhs_src)?;
        let rhs = parse_expression(&rhs_src)?;
        return Ok(equation_piece(label, lhs_src, rhs_src, lhs, rhs));
    }
    if let Some((order, body)) = parse_derivative_body(&normalized)? {
        return calculus_derivative(&label, &body, order, YSide::None);
    }
    if let Some(query) = parse_integral_query(&normalized)? {
        return dispatch_integral(&label, query, YSide::None);
    }
    let expr = parse_expression(&normalized)?;
    if expr.uses_x() {
        Ok(function_piece(label, normalized, &expr))
    } else {
        Ok(value_piece(label, &expr))
    }
}

fn dispatch_integral(label: &str, query: IntegralQuery, side: YSide) -> Result<MathPiece, String> {
    match (query.lower, query.upper) {
        (Some(lower), Some(upper)) => calculus_definite(label, &query.body, &lower, &upper, side),
        (None, None) => calculus_integral(label, &query.body, side),
        _ => Err("An integral with bounds needs both a lower and an upper limit".into()),
    }
}

fn split_equation(input: &str) -> Option<(String, String)> {
    let mut depth = 0i32;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' | '{' => depth += 1,
            ')' | '}' => depth -= 1,
            '=' if depth == 0 => {
                let lhs = input[..index].trim().to_string();
                let rhs = input[index + 1..].trim().to_string();
                return Some((lhs, rhs));
            }
            _ => {}
        }
    }
    None
}

fn value_piece(label: String, expr: &Expr) -> MathPiece {
    let value = eval(expr, None).unwrap_or(f64::NAN);
    let result = number_latex(value);
    MathPiece {
        expression: label.clone(),
        latex: latex_expr(expr),
        uses_x: false,
        value: Some(value),
        result_label: "Result".into(),
        result_plain: format_number(value),
        result_latex: result,
        note: None,
        plots: Vec::new(),
        solutions: Vec::new(),
        markers: Vec::new(),
        frame: None,
        label,
    }
}

fn function_piece(label: String, source: String, expr: &Expr) -> MathPiece {
    let body = latex_expr(expr);
    MathPiece {
        expression: source.clone(),
        latex: format!("y = {body}"),
        uses_x: expr.uses_x(),
        value: if expr.uses_x() {
            None
        } else {
            Some(eval(expr, None).unwrap_or(f64::NAN))
        },
        result_label: String::new(),
        result_plain: String::new(),
        result_latex: String::new(),
        note: None,
        plots: vec![PlotSpec {
            label: label.clone(),
            expression: source,
            plot: "function".into(),
        }],
        solutions: Vec::new(),
        markers: Vec::new(),
        frame: None,
        label,
    }
}

fn equation_piece(label: String, lhs_src: String, rhs_src: String, lhs: Expr, rhs: Expr) -> MathPiece {
    let latex = format!("{} = {}", latex_expr(&lhs), latex_expr(&rhs));
    if is_x(&lhs) && !rhs.uses_x() {
        return vertical_piece(label, latex, eval(&rhs, None).unwrap_or(f64::NAN));
    }
    if is_x(&rhs) && !lhs.uses_x() {
        return vertical_piece(label, latex, eval(&lhs, None).unwrap_or(f64::NAN));
    }
    if !lhs.uses_x() && !rhs.uses_x() {
        let left = eval(&lhs, None).unwrap_or(f64::NAN);
        let right = eval(&rhs, None).unwrap_or(f64::NAN);
        let holds = left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-8 * (1.0 + left.abs());
        return MathPiece {
            expression: lhs_src.clone(),
            latex,
            uses_x: false,
            value: None,
            result_label: "Result".into(),
            result_plain: if holds { "True".into() } else { "False".into() },
            result_latex: if holds {
                "\\text{true}".into()
            } else {
                "\\text{false}".into()
            },
            note: None,
            plots: vec![
                PlotSpec {
                    label: lhs_src.clone(),
                    expression: lhs_src,
                    plot: "function".into(),
                },
                PlotSpec {
                    label: rhs_src.clone(),
                    expression: rhs_src,
                    plot: "function".into(),
                },
            ],
            solutions: Vec::new(),
            markers: Vec::new(),
            frame: None,
            label,
        };
    }

    let diff = Expr::Sub(Box::new(lhs.clone()), Box::new(rhs.clone()));
    let solved = solve_real(&diff);
    let (result_label, result_plain, result_latex, note, markers, frame) = match &solved {
        Solved::All => (
            "Result".into(),
            "True for all real x".into(),
            "\\text{true for all real } x".into(),
            None,
            Vec::new(),
            None,
        ),
        Solved::None => (
            "Solutions".into(),
            "No real solution".into(),
            "\\text{no real solution}".into(),
            None,
            Vec::new(),
            None,
        ),
        Solved::Roots { values, bounded } => {
            let solutions: Vec<Solution> = values.iter().copied().map(solution_at).collect();
            let markers = values
                .iter()
                .filter_map(|&root| {
                    let y = eval(&rhs, Some(root)).ok().filter(|value| value.is_finite())?;
                    Some(Marker { x: root, y })
                })
                .collect::<Vec<_>>();
            let note = bounded.then(|| "Real solutions on the interval [-20, 20].".to_string());
            let frame = frame_around(
                &markers.iter().map(|marker| marker.x).collect::<Vec<_>>(),
                &markers
                    .iter()
                    .map(|marker| marker.y)
                    .chain([0.0])
                    .collect::<Vec<_>>(),
            );
            let result_plain = solutions
                .iter()
                .map(|solution| solution.plain.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let result_latex = solutions_latex(&solutions);
            let result_label = if solutions.len() == 1 {
                "Solution".into()
            } else {
                "Solutions".into()
            };
            (result_label, result_plain, result_latex, note, markers, Some(frame))
        }
    };
    MathPiece {
        expression: lhs_src.clone(),
        latex,
        uses_x: true,
        value: None,
        result_label,
        result_plain,
        result_latex,
        note,
        plots: vec![
            PlotSpec {
                label: lhs_src.clone(),
                expression: lhs_src,
                plot: "function".into(),
            },
            PlotSpec {
                label: rhs_src.clone(),
                expression: rhs_src,
                plot: "function".into(),
            },
        ],
        solutions: match solved {
            Solved::Roots { values, .. } => values.into_iter().map(solution_at).collect(),
            _ => Vec::new(),
        },
        markers,
        frame,
        label,
    }
}

fn vertical_piece(label: String, latex: String, x_value: f64) -> MathPiece {
    let shown = number_latex(x_value);
    let plain = format!("x = {}", format_number(x_value));
    let x_text = format_number(x_value);
    let x_num = x_text.parse::<f64>().unwrap_or(x_value);
    MathPiece {
        expression: x_text.clone(),
        latex,
        uses_x: true,
        value: None,
        result_label: "Solution".into(),
        result_plain: plain.clone(),
        result_latex: format!("x = {shown}"),
        note: None,
        plots: vec![PlotSpec {
            label: label.clone(),
            expression: x_text,
            plot: "vertical".into(),
        }],
        solutions: vec![Solution {
            plain,
            latex: format!("x = {shown}"),
        }],
        markers: Vec::new(),
        frame: Some(frame_around(&[x_num], &[0.0])),
        label,
    }
}

fn is_x(expr: &Expr) -> bool {
    matches!(expr, Expr::Var)
}

fn solution_at(root: f64) -> Solution {
    let shown = number_latex(root);
    Solution {
        plain: format!("x = {}", format_number(root)),
        latex: format!("x = {shown}"),
    }
}

fn solutions_latex(solutions: &[Solution]) -> String {
    if solutions.len() == 1 {
        return solutions[0].latex.clone();
    }
    let lines = solutions
        .iter()
        .map(|solution| solution.latex.replacen(" = ", " &= ", 1))
        .collect::<Vec<_>>()
        .join(" \\\\ ");
    format!("\\begin{{aligned}} {lines} \\end{{aligned}}")
}

fn frame_around(xs: &[f64], ys: &[f64]) -> (f64, f64, f64, f64) {
    let (x0, x1) = span(xs, -6.0, 6.0);
    let (y0, y1) = span(ys, -6.0, 6.0);
    let xpad = ((x1 - x0) * 0.5).max(4.0);
    let ypad = ((y1 - y0) * 0.5).max(3.0);
    (x0 - xpad, x1 + xpad, y0 - ypad, y1 + ypad)
}

fn span(values: &[f64], fallback_lo: f64, fallback_hi: f64) -> (f64, f64) {
    let finite: Vec<f64> = values.iter().copied().filter(|value| value.is_finite()).collect();
    if finite.is_empty() {
        return (fallback_lo, fallback_hi);
    }
    let lo = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (lo, hi)
}

fn split_pieces(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ';' if depth == 0 => {
                let piece = input[start..index].trim();
                if !piece.is_empty() {
                    parts.push(piece.to_string());
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let tail = input[start..].trim();
    if !tail.is_empty() {
        parts.push(tail.to_string());
    }
    parts
}

fn name_cut(word: &str, bounds: &[String]) -> usize {
    if word.is_empty() {
        return 0;
    }
    for len in (1..=word.len()).rev() {
        if is_known_name(&word[..len], bounds) {
            return len;
        }
    }
    1
}

fn is_known_name(name: &str, bounds: &[String]) -> bool {
    if bounds.iter().any(|bound| bound == name) {
        return true;
    }
    matches!(
        name,
        "x" | "pi" | "e" | "i" | "inf" | "infinity" | "oo" | "theta" | "sum" | "prod" | "product" | "lim" | "limit" | "int" | "integral"
    )     || Func::from_name(name).is_some()
}

fn parse_derivative_body(input: &str) -> Result<Option<(u32, String)>, String> {
    let Some((order, rest)) = strip_derivative_prefix(input.trim()) else {
        return Ok(None);
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Err("Write a derivative as d/dx f(x)".into());
    }
    Ok(Some((order, rest.to_string())))
}

fn strip_derivative_prefix(input: &str) -> Option<(u32, &str)> {
    let mut index = skip_ascii_ws(input, 0);
    index = eat_ascii_token(input, index, "d", false)?;
    let (mut index, n1) = eat_optional_power(input, index).ok()?;
    index = skip_ascii_ws(input, index);
    index = eat_ascii_token(input, index, "/", true)?;
    index = skip_ascii_ws(input, index);
    if let Some(after_dx) = eat_ascii_token(input, index, "dx", true) {
        if !input[after_dx..].trim_start().starts_with('^') {
            let order = n1.unwrap_or(1);
            if (1..=6).contains(&order) {
                return Some((order, &input[after_dx..]));
            }
        }
    }
    index = eat_ascii_token(input, index, "d", true)?;
    index = skip_ascii_ws(input, index);
    index = eat_ascii_token(input, index, "x", true)?;
    let (index, n2) = eat_optional_power(input, index).ok()?;
    let order = match (n1, n2) {
        (Some(left), Some(right)) if left == right => left,
        (Some(order), None) | (None, Some(order)) => order,
        (None, None) => 1,
        _ => return None,
    };
    (1..=6).contains(&order).then_some((order, &input[index..]))
}

fn eat_optional_power(input: &str, index: usize) -> Result<(usize, Option<u32>), ()> {
    let skipped = skip_ascii_ws(input, index);
    if !input[skipped..].starts_with('^') {
        return Ok((index, None));
    }
    let mut cursor = skipped + 1;
    cursor = skip_ascii_ws(input, cursor);
    let rest = input.get(cursor..).ok_or(())?;
    let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Err(());
    }
    let order = digits.parse::<u32>().map_err(|_| ())?;
    Ok((cursor + digits.len(), Some(order)))
}

fn parse_integral_query(input: &str) -> Result<Option<IntegralQuery>, String> {
    let Some(rest) = strip_integral_keyword(input.trim()) else {
        return Ok(None);
    };
    let (rest, from_bounds) = peel_from_to(rest);
    let rest = rest.trim();
    if rest.is_empty() {
        return Err("Write an integral as int(f(x)) dx".into());
    }
    let (lower, upper, rest) = if let Some((lower, upper, rest)) = take_underscore_bounds(rest) {
        (Some(lower), Some(upper), rest.trim())
    } else if let Some((lower, upper)) = from_bounds {
        (Some(lower), Some(upper), rest)
    } else {
        (None, None, rest)
    };
    let body = integrand_source(rest)?;
    Ok(Some(IntegralQuery { body, lower, upper }))
}

fn integrand_source(rest: &str) -> Result<String, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Err("Write an integral as int(f(x)) dx".into());
    }
    if let Some((inside, after)) = take_balanced_paren(rest) {
        let after = after.trim();
        if is_differential(after) {
            let inside = inside.trim();
            if inside.is_empty() {
                return Err("Write an integral as int(f(x)) dx".into());
            }
            return Ok(inside.to_string());
        }
        if after.is_empty() {
            if let Some(body) = strip_differential(inside) {
                return Ok(body.to_string());
            }
        }
        return Err("Write an integral as int(f(x)) dx".into());
    }
    match strip_differential(rest) {
        Some(body) if !body.is_empty() => Ok(body.to_string()),
        _ => Err("Write an integral as int(f(x)) dx".into()),
    }
}

fn peel_from_to(input: &str) -> (&str, Option<(String, String)>) {
    let lower_input = input.to_ascii_lowercase();
    let mut depth = 0i32;
    let mut index = 0usize;
    while index < input.len() {
        let ch = input[index..].chars().next().unwrap();
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 && lower_input[index..].starts_with(" from ") => {
                let after = index + " from ".len();
                if let Some(to_at) = lower_input[after..].find(" to ") {
                    let lower = input[after..after + to_at].trim();
                    let upper = input[after + to_at + " to ".len()..].trim();
                    if !lower.is_empty() && !upper.is_empty() {
                        return (input[..index].trim(), Some((lower.to_string(), upper.to_string())));
                    }
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    (input, None)
}

fn take_underscore_bounds(input: &str) -> Option<(String, String, &str)> {
    let input = input.trim_start();
    if !input.starts_with('_') {
        return None;
    }
    let (lower, after_lower) = take_bound(&input[1..])?;
    let after_lower = after_lower.trim_start();
    if !after_lower.starts_with('^') {
        return None;
    }
    let (upper, rest) = take_bound(after_lower[1..].trim_start())?;
    Some((lower, upper, rest))
}

fn take_bound(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    if let Some(rest) = input.strip_prefix('{') {
        let mut depth = 1i32;
        for (index, ch) in rest.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let inside = rest[..index].trim();
                        if inside.is_empty() {
                            return None;
                        }
                        let after = index + ch.len_utf8();
                        return Some((inside.to_string(), &rest[after..]));
                    }
                }
                _ => {}
            }
        }
        return None;
    }
    if input.starts_with('(') {
        let (inside, rest) = take_balanced_paren(input)?;
        return Some((inside.trim().to_string(), rest));
    }
    let mut end = 0usize;
    let bytes = input.as_bytes();
    if bytes.first().is_some_and(|byte| *byte == b'+' || *byte == b'-') {
        end = 1;
    }
    let content = end;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'.') {
        end += 1;
    }
    if end <= content {
        return None;
    }
    Some((input[..end].to_string(), &input[end..]))
}

fn strip_integral_keyword(input: &str) -> Option<&str> {
    if let Some(rest) = input.strip_prefix('∫') {
        return Some(rest);
    }
    let lower = input.to_ascii_lowercase();
    for key in ["integral", "int"] {
        if let Some(rest) = lower.strip_prefix(key) {
            if rest.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
                return None;
            }
            return Some(&input[key.len()..]);
        }
    }
    None
}

fn take_balanced_paren(input: &str) -> Option<(&str, &str)> {
    if !input.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let end = index + ch.len_utf8();
                    return Some((&input[1..index], &input[end..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn is_differential(input: &str) -> bool {
    let compact: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();
    compact.eq_ignore_ascii_case("dx")
}

fn strip_differential(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.ends_with("dx") || trimmed.len() < 3 {
        return None;
    }
    let cut = trimmed.len() - 2;
    let before = trimmed[..cut].chars().next_back()?;
    if before.is_ascii_alphabetic() {
        return None;
    }
    let body = trimmed[..cut].trim();
    if body.is_empty() {
        None
    } else {
        Some(body)
    }
}

fn skip_ascii_ws(input: &str, mut index: usize) -> usize {
    while input[index..].starts_with(|ch: char| ch.is_whitespace()) {
        index += 1;
    }
    index
}

fn eat_ascii_token(input: &str, index: usize, token: &str, glue_ok: bool) -> Option<usize> {
    let rest = input.get(index..)?;
    if rest.len() < token.len() || !rest[..token.len()].eq_ignore_ascii_case(token) {
        return None;
    }
    let next = rest[token.len()..].chars().next();
    if !glue_ok && token.chars().all(|ch| ch.is_ascii_alphabetic()) && next.is_some_and(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(index + token.len())
}

fn calculus_derivative(label: &str, body: &str, order: u32, side: YSide) -> Result<MathPiece, String> {
    let expr = parse_expression(body)?;
    let mut derived = expr.clone();
    for _ in 0..order {
        derived = differentiate(&derived);
    }
    let derived = simplify(derived);
    Ok(finish_calculus(label, &expr, &derived, false, order, side, None))
}

fn calculus_integral(label: &str, body: &str, side: YSide) -> Result<MathPiece, String> {
    let expr = parse_expression(body)?;
    let integrated = simplify(integrate(&expr)?);
    Ok(finish_calculus(label, &expr, &integrated, true, 1, side, None))
}

fn calculus_definite(
    label: &str,
    body: &str,
    lower: &str,
    upper: &str,
    side: YSide,
) -> Result<MathPiece, String> {
    let expr = parse_expression(body)?;
    let integrated = simplify(integrate(&expr)?);
    let low = eval_bound(lower)?;
    let high = eval_bound(upper)?;
    let value = eval(&integrated, Some(high))? - eval(&integrated, Some(low))?;
    if !value.is_finite() {
        return Err("That definite integral is undefined".into());
    }
    let mut piece = finish_calculus(label, &expr, &expr, true, 1, side, Some((lower, upper, value)));
    piece.expression = expr_to_source(&expr);
    if let Some(plot) = piece.plots.first_mut() {
        plot.expression = piece.expression.clone();
    }
    piece.uses_x = expr.uses_x();
    piece.value = Some(value);
    piece.result_plain = format_number(value);
    piece.result_latex = if side == YSide::None {
        number_latex(value)
    } else {
        format!("y = {}", number_latex(value))
    };
    piece.note = Some("Definite integral.".into());
    Ok(piece)
}

fn eval_bound(source: &str) -> Result<f64, String> {
    let expr = parse_expression(source)?;
    if expr.uses_x() {
        return Err("Integral bounds should be constants".into());
    }
    let value = eval(&expr, None)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err("That bound is undefined".into())
    }
}

fn finish_calculus(
    label: &str,
    input: &Expr,
    result: &Expr,
    integral: bool,
    order: u32,
    side: YSide,
    bounds: Option<(&str, &str, f64)>,
) -> MathPiece {
    let source = expr_to_source(result);
    let pretty = latex_expr(result);
    let plain = expr_to_plain(result);
    let operand = latex_operand(input);
    let (latex, result_latex, result_plain, result_label, note) = if integral {
        let operation = match bounds {
            Some((lower, upper, _)) => {
                let lower = bound_latex(lower);
                let upper = bound_latex(upper);
                format!("\\int_{{{lower}}}^{{{upper}}} {operand}\\,dx")
            }
            None => format!("\\int {operand}\\,dx"),
        };
        let note = bounds.is_none().then(|| "Indefinite integral. The graph uses C = 0.".to_string());
        let shown = if bounds.is_some() { pretty.clone() } else { format!("{pretty} + C") };
        let shown_plain = if bounds.is_some() { plain.clone() } else { format!("{plain} + C") };
        match side {
            YSide::Left => (
                format!("y = {operation}"),
                format!("y = {shown}"),
                format!("y = {shown_plain}"),
                "Integral",
                note,
            ),
            YSide::Right => (
                format!("{operation} = y"),
                format!("y = {shown}"),
                format!("y = {shown_plain}"),
                "Integral",
                note,
            ),
            YSide::None => (operation, shown, shown_plain, "Integral", note),
        }
    } else {
        let operation = if order <= 1 {
            format!("\\frac{{d}}{{dx}}{operand}")
        } else {
            format!("\\frac{{d^{{{order}}}}}{{dx^{{{order}}}}}{operand}")
        };
        match side {
            YSide::Left => (
                format!("y = {operation}"),
                format!("y = {pretty}"),
                format!("y = {plain}"),
                "Derivative",
                None,
            ),
            YSide::Right => (
                format!("{operation} = y"),
                format!("y = {pretty}"),
                format!("y = {plain}"),
                "Derivative",
                None,
            ),
            YSide::None => (operation, pretty, plain, "Derivative", None),
        }
    };
    let uses_x = result.uses_x();
    let value = if uses_x {
        None
    } else {
        Some(eval(result, None).unwrap_or(f64::NAN))
    };
    MathPiece {
        expression: source.clone(),
        latex,
        uses_x,
        value,
        result_label: result_label.into(),
        result_plain,
        result_latex,
        note,
        plots: vec![PlotSpec {
            label: label.to_string(),
            expression: source,
            plot: "function".into(),
        }],
        solutions: Vec::new(),
        markers: Vec::new(),
        frame: None,
        label: label.to_string(),
    }
}

fn bound_latex(source: &str) -> String {
    parse_expression(source)
        .map(|expr| latex_expr(&expr))
        .unwrap_or_else(|_| source.to_string())
}

fn latex_operand(expr: &Expr) -> String {
    match expr {
        Expr::Add(_, _) | Expr::Sub(_, _) => format!("\\left({}\\right)", latex_expr(expr)),
        _ => latex_expr(expr),
    }
}

fn differentiate(expr: &Expr) -> Expr {
    match expr {
        Expr::Num(_) => Expr::Num(0.0),
        Expr::Var => Expr::Num(1.0),
        Expr::Neg(inner) => Expr::Neg(Box::new(differentiate(inner))),
        Expr::Add(left, right) => Expr::Add(Box::new(differentiate(left)), Box::new(differentiate(right))),
        Expr::Sub(left, right) => Expr::Sub(Box::new(differentiate(left)), Box::new(differentiate(right))),
        Expr::Mul(left, right) => Expr::Add(
            Box::new(Expr::Mul(Box::new(differentiate(left)), right.clone())),
            Box::new(Expr::Mul(left.clone(), Box::new(differentiate(right)))),
        ),
        Expr::Div(left, right) => Expr::Div(
            Box::new(Expr::Sub(
                Box::new(Expr::Mul(Box::new(differentiate(left)), right.clone())),
                Box::new(Expr::Mul(left.clone(), Box::new(differentiate(right)))),
            )),
            Box::new(Expr::Pow(right.clone(), Box::new(Expr::Num(2.0)))),
        ),
        Expr::Pow(base, exp) if !exp.uses_x() => Expr::Mul(
            Box::new(Expr::Mul(
                exp.clone(),
                Box::new(Expr::Pow(base.clone(), Box::new(Expr::Sub(exp.clone(), Box::new(Expr::Num(1.0)))))),
            )),
            Box::new(differentiate(base)),
        ),
        Expr::Pow(base, exp) => {
            let extra = Expr::Add(
                Box::new(Expr::Mul(
                    Box::new(differentiate(exp)),
                    Box::new(Expr::Call(Func::Ln, base.clone())),
                )),
                Box::new(Expr::Mul(
                    exp.clone(),
                    Box::new(Expr::Div(Box::new(differentiate(base)), base.clone())),
                )),
            );
            Expr::Mul(Box::new(expr.clone()), Box::new(extra))
        }
        Expr::Call(func, arg) => differentiate_call(*func, arg, differentiate(arg)),
        Expr::Index(_) => Expr::Num(0.0),
        Expr::Sum { var, lower, upper, body } => Expr::Sum {
            var: var.clone(),
            lower: lower.clone(),
            upper: upper.clone(),
            body: Box::new(differentiate(body)),
        },
        Expr::Prod { var, lower, upper, body } => Expr::Prod {
            var: var.clone(),
            lower: lower.clone(),
            upper: upper.clone(),
            body: Box::new(differentiate(body)),
        },
        Expr::Limit { var, target, body } => Expr::Limit {
            var: var.clone(),
            target: target.clone(),
            body: Box::new(differentiate(body)),
        },
    }
}

fn differentiate_call(func: Func, arg: &Expr, darg: Expr) -> Expr {
    let chain = |outer: Expr| Expr::Mul(Box::new(outer), Box::new(darg.clone()));
    let call = |func: Func, inner: Expr| Expr::Call(func, Box::new(inner));
    let num = |value: f64| Expr::Num(value);
    match func {
        Func::Sin => chain(call(Func::Cos, arg.clone())),
        Func::Cos => chain(Expr::Neg(Box::new(call(Func::Sin, arg.clone())))),
        Func::Tan => chain(Expr::Pow(
            Box::new(call(Func::Sec, arg.clone())),
            Box::new(num(2.0)),
        )),
        Func::Sec => chain(Expr::Mul(
            Box::new(call(Func::Sec, arg.clone())),
            Box::new(call(Func::Tan, arg.clone())),
        )),
        Func::Csc => chain(Expr::Neg(Box::new(Expr::Mul(
            Box::new(call(Func::Csc, arg.clone())),
            Box::new(call(Func::Cot, arg.clone())),
        )))),
        Func::Cot => chain(Expr::Neg(Box::new(Expr::Pow(
            Box::new(call(Func::Csc, arg.clone())),
            Box::new(num(2.0)),
        )))),
        Func::Asin => Expr::Div(
            Box::new(darg),
            Box::new(call(
                Func::Sqrt,
                Expr::Sub(Box::new(num(1.0)), Box::new(Expr::Pow(Box::new(arg.clone()), Box::new(num(2.0))))),
            )),
        ),
        Func::Acos | Func::Acot => Expr::Neg(Box::new(differentiate_call(
            if func == Func::Acos { Func::Asin } else { Func::Atan },
            arg,
            darg,
        ))),
        Func::Atan => Expr::Div(
            Box::new(darg),
            Box::new(Expr::Add(
                Box::new(num(1.0)),
                Box::new(Expr::Pow(Box::new(arg.clone()), Box::new(num(2.0)))),
            )),
        ),
        Func::Asec => Expr::Div(
            Box::new(darg),
            Box::new(Expr::Mul(
                Box::new(call(Func::Abs, arg.clone())),
                Box::new(call(
                    Func::Sqrt,
                    Expr::Sub(Box::new(Expr::Pow(Box::new(arg.clone()), Box::new(num(2.0)))), Box::new(num(1.0))),
                )),
            )),
        ),
        Func::Acsc => Expr::Neg(Box::new(differentiate_call(Func::Asec, arg, darg))),
        Func::Sinh => chain(call(Func::Cosh, arg.clone())),
        Func::Cosh => chain(call(Func::Sinh, arg.clone())),
        Func::Tanh => chain(Expr::Pow(Box::new(call(Func::Sech, arg.clone())), Box::new(num(2.0)))),
        Func::Sech => chain(Expr::Neg(Box::new(Expr::Mul(
            Box::new(call(Func::Sech, arg.clone())),
            Box::new(call(Func::Tanh, arg.clone())),
        )))),
        Func::Csch => chain(Expr::Neg(Box::new(Expr::Mul(
            Box::new(call(Func::Csch, arg.clone())),
            Box::new(call(Func::Coth, arg.clone())),
        )))),
        Func::Coth => chain(Expr::Neg(Box::new(Expr::Pow(
            Box::new(call(Func::Csch, arg.clone())),
            Box::new(num(2.0)),
        )))),
        Func::Asinh => Expr::Div(
            Box::new(darg),
            Box::new(call(
                Func::Sqrt,
                Expr::Add(Box::new(Expr::Pow(Box::new(arg.clone()), Box::new(num(2.0)))), Box::new(num(1.0))),
            )),
        ),
        Func::Acosh => Expr::Div(
            Box::new(darg),
            Box::new(call(
                Func::Sqrt,
                Expr::Sub(Box::new(Expr::Pow(Box::new(arg.clone()), Box::new(num(2.0)))), Box::new(num(1.0))),
            )),
        ),
        Func::Atanh => Expr::Div(
            Box::new(darg),
            Box::new(Expr::Sub(
                Box::new(num(1.0)),
                Box::new(Expr::Pow(Box::new(arg.clone()), Box::new(num(2.0)))),
            )),
        ),
        Func::Ln => Expr::Div(Box::new(darg), Box::new(arg.clone())),
        Func::Log => Expr::Div(
            Box::new(darg),
            Box::new(Expr::Mul(Box::new(arg.clone()), Box::new(call(Func::Ln, num(10.0))))),
        ),
        Func::Log2 => Expr::Div(
            Box::new(darg),
            Box::new(Expr::Mul(Box::new(arg.clone()), Box::new(call(Func::Ln, num(2.0))))),
        ),
        Func::Sqrt => Expr::Div(
            Box::new(darg),
            Box::new(Expr::Mul(Box::new(num(2.0)), Box::new(call(Func::Sqrt, arg.clone())))),
        ),
        Func::Abs => Expr::Mul(
            Box::new(darg),
            Box::new(Expr::Div(Box::new(arg.clone()), Box::new(call(Func::Abs, arg.clone())))),
        ),
        Func::Floor | Func::Ceil | Func::Round | Func::Sign | Func::Trigamma => num(0.0),
        Func::Fact => chain(Expr::Mul(
            Box::new(call(Func::Fact, arg.clone())),
            Box::new(call(Func::Digamma, Expr::Add(Box::new(arg.clone()), Box::new(num(1.0))))),
        )),
        Func::Gamma => chain(Expr::Mul(
            Box::new(call(Func::Gamma, arg.clone())),
            Box::new(call(Func::Digamma, arg.clone())),
        )),
        Func::Digamma => chain(call(Func::Trigamma, arg.clone())),
    }
}

fn integrate(expr: &Expr) -> Result<Expr, String> {
    let expr = simplify(expr.clone());
    if !expr.uses_x() {
        let value = eval(&expr, None).map_err(|_| "No symbolic integral for that yet".to_string())?;
        if !value.is_finite() {
            return Err("No symbolic integral for that yet".into());
        }
        return Ok(Expr::Mul(Box::new(Expr::Num(value)), Box::new(Expr::Var)));
    }
    match &expr {
        Expr::Var => Ok(Expr::Div(
            Box::new(Expr::Pow(Box::new(Expr::Var), Box::new(Expr::Num(2.0)))),
            Box::new(Expr::Num(2.0)),
        )),
        Expr::Neg(inner) => Ok(Expr::Neg(Box::new(integrate(inner)?))),
        Expr::Add(left, right) => Ok(Expr::Add(Box::new(integrate(left)?), Box::new(integrate(right)?))),
        Expr::Sub(left, right) => Ok(Expr::Sub(Box::new(integrate(left)?), Box::new(integrate(right)?))),
        Expr::Mul(left, right) if !left.uses_x() => Ok(Expr::Mul(left.clone(), Box::new(integrate(right)?))),
        Expr::Mul(left, right) if !right.uses_x() => Ok(Expr::Mul(right.clone(), Box::new(integrate(left)?))),
        Expr::Div(left, right) if is_one(left) && matches!(right.as_ref(), Expr::Var) => {
            Ok(Expr::Call(Func::Ln, Box::new(Expr::Call(Func::Abs, Box::new(Expr::Var)))))
        }
        Expr::Div(left, right) if !right.uses_x() => Ok(Expr::Div(Box::new(integrate(left)?), right.clone())),
        Expr::Pow(base, exp) if is_e(base) && matches!(exp.as_ref(), Expr::Var) => Ok(expr),
        Expr::Pow(base, exp) if matches!(base.as_ref(), Expr::Var) && !exp.uses_x() => {
            let power = const_value(exp).ok_or("No symbolic integral for that yet")?;
            if (power + 1.0).abs() < 1e-9 {
                return Ok(Expr::Call(Func::Ln, Box::new(Expr::Call(Func::Abs, Box::new(Expr::Var)))));
            }
            Ok(Expr::Div(
                Box::new(Expr::Pow(Box::new(Expr::Var), Box::new(Expr::Num(power + 1.0)))),
                Box::new(Expr::Num(power + 1.0)),
            ))
        }
        Expr::Call(Func::Sin, arg) => {
            let (slope, _) = linear_coeff(arg).ok_or("No symbolic integral for that yet")?;
            if slope.abs() < 1e-12 {
                return Err("No symbolic integral for that yet".into());
            }
            Ok(Expr::Div(
                Box::new(Expr::Neg(Box::new(Expr::Call(Func::Cos, arg.clone())))),
                Box::new(Expr::Num(slope)),
            ))
        }
        Expr::Call(Func::Cos, arg) => {
            let (slope, _) = linear_coeff(arg).ok_or("No symbolic integral for that yet")?;
            if slope.abs() < 1e-12 {
                return Err("No symbolic integral for that yet".into());
            }
            Ok(Expr::Div(
                Box::new(Expr::Call(Func::Sin, arg.clone())),
                Box::new(Expr::Num(slope)),
            ))
        }
        _ => Err("No symbolic integral for that yet".into()),
    }
}

fn linear_coeff(expr: &Expr) -> Option<(f64, f64)> {
    match expr {
        Expr::Var => Some((1.0, 0.0)),
        Expr::Num(value) => Some((0.0, *value)),
        Expr::Neg(inner) => {
            let (slope, intercept) = linear_coeff(inner)?;
            Some((-slope, -intercept))
        }
        Expr::Mul(left, right) if !left.uses_x() && matches!(right.as_ref(), Expr::Var) => {
            Some((const_value(left)?, 0.0))
        }
        Expr::Mul(left, right) if !right.uses_x() && matches!(left.as_ref(), Expr::Var) => {
            Some((const_value(right)?, 0.0))
        }
        Expr::Add(left, right) => {
            let (slope, intercept) = linear_coeff(left)?;
            let (slope_right, intercept_right) = linear_coeff(right)?;
            Some((slope + slope_right, intercept + intercept_right))
        }
        Expr::Sub(left, right) => {
            let (slope, intercept) = linear_coeff(left)?;
            let (slope_right, intercept_right) = linear_coeff(right)?;
            Some((slope - slope_right, intercept - intercept_right))
        }
        _ => None,
    }
}

fn const_value(expr: &Expr) -> Option<f64> {
    if expr.uses_x() {
        return None;
    }
    let value = eval(expr, None).ok()?;
    value.is_finite().then_some(value)
}

fn is_one(expr: &Expr) -> bool {
    matches!(expr, Expr::Num(value) if (value - 1.0).abs() < 1e-9)
}

fn is_e(expr: &Expr) -> bool {
    matches!(expr, Expr::Num(value) if (value - E).abs() < 1e-8)
}

fn simplify(expr: Expr) -> Expr {
    let mut expr = expr;
    for _ in 0..16 {
        let (next, changed) = rewrite_once(expr);
        expr = next;
        if !changed {
            break;
        }
    }
    expr
}

fn rewrite_once(expr: Expr) -> (Expr, bool) {
    let (expr, child_changed) = match expr {
        Expr::Num(value) => return (Expr::Num(value), false),
        Expr::Var => return (Expr::Var, false),
        Expr::Index(name) => return (Expr::Index(name), false),
        Expr::Sum { var, lower, upper, body } => {
            let (lower, left) = rewrite_once(*lower);
            let (upper, right) = rewrite_once(*upper);
            let (body, inner) = rewrite_once(*body);
            return (
                Expr::Sum {
                    var,
                    lower: Box::new(lower),
                    upper: Box::new(upper),
                    body: Box::new(body),
                },
                left || right || inner,
            );
        }
        Expr::Prod { var, lower, upper, body } => {
            let (lower, left) = rewrite_once(*lower);
            let (upper, right) = rewrite_once(*upper);
            let (body, inner) = rewrite_once(*body);
            return (
                Expr::Prod {
                    var,
                    lower: Box::new(lower),
                    upper: Box::new(upper),
                    body: Box::new(body),
                },
                left || right || inner,
            );
        }
        Expr::Limit { var, target, body } => {
            let (target, left) = rewrite_once(*target);
            let (body, inner) = rewrite_once(*body);
            return (
                Expr::Limit {
                    var,
                    target: Box::new(target),
                    body: Box::new(body),
                },
                left || inner,
            );
        }
        Expr::Neg(inner) => {
            let (inner, changed) = rewrite_once(*inner);
            (Expr::Neg(Box::new(inner)), changed)
        }
        Expr::Add(left, right) => map_binary(left, right, Expr::Add),
        Expr::Sub(left, right) => map_binary(left, right, Expr::Sub),
        Expr::Mul(left, right) => map_binary(left, right, Expr::Mul),
        Expr::Div(left, right) => map_binary(left, right, Expr::Div),
        Expr::Pow(left, right) => map_binary(left, right, Expr::Pow),
        Expr::Call(func, arg) => {
            let (arg, changed) = rewrite_once(*arg);
            (Expr::Call(func, Box::new(arg)), changed)
        }
    };
    let (rewritten, local) = rewrite_local(expr);
    (rewritten, child_changed || local)
}

fn map_binary(
    left: Box<Expr>,
    right: Box<Expr>,
    build: fn(Box<Expr>, Box<Expr>) -> Expr,
) -> (Expr, bool) {
    let (left, left_changed) = rewrite_once(*left);
    let (right, right_changed) = rewrite_once(*right);
    (build(Box::new(left), Box::new(right)), left_changed || right_changed)
}

fn rewrite_local(expr: Expr) -> (Expr, bool) {
    let zero = |value: f64| value.abs() < 1e-12;
    let one = |value: f64| (value - 1.0).abs() < 1e-12;
    let neg_one = |value: f64| (value + 1.0).abs() < 1e-12;
    match expr {
        Expr::Neg(inner) => match *inner {
            Expr::Num(value) => (Expr::Num(-value), true),
            Expr::Neg(inner) => (*inner, true),
            other => (Expr::Neg(Box::new(other)), false),
        },
        Expr::Add(left, right) => match (left.as_ref(), right.as_ref()) {
            (Expr::Num(a), Expr::Num(b)) => (Expr::Num(a + b), true),
            (Expr::Num(value), _) if zero(*value) => (*right, true),
            (_, Expr::Num(value)) if zero(*value) => (*left, true),
            _ => (Expr::Add(left, right), false),
        },
        Expr::Sub(left, right) => match (left.as_ref(), right.as_ref()) {
            (Expr::Num(a), Expr::Num(b)) => (Expr::Num(a - b), true),
            (_, Expr::Num(value)) if zero(*value) => (*left, true),
            (Expr::Num(value), _) if zero(*value) => (Expr::Neg(right), true),
            _ => (Expr::Sub(left, right), false),
        },
        Expr::Mul(left, right) => match (left.as_ref(), right.as_ref()) {
            (Expr::Num(a), Expr::Num(b)) => (Expr::Num(a * b), true),
            (Expr::Num(value), _) if zero(*value) => (Expr::Num(0.0), true),
            (_, Expr::Num(value)) if zero(*value) => (Expr::Num(0.0), true),
            (Expr::Num(value), _) if one(*value) => (*right, true),
            (_, Expr::Num(value)) if one(*value) => (*left, true),
            (Expr::Num(value), _) if neg_one(*value) => (Expr::Neg(right), true),
            (_, Expr::Num(value)) if neg_one(*value) => (Expr::Neg(left), true),
            (Expr::Neg(inner), Expr::Neg(other)) => (Expr::Mul(inner.clone(), other.clone()), true),
            (Expr::Neg(inner), _) => (Expr::Neg(Box::new(Expr::Mul(inner.clone(), right))), true),
            (_, Expr::Neg(inner)) => (Expr::Neg(Box::new(Expr::Mul(left, inner.clone()))), true),
            _ => (Expr::Mul(left, right), false),
        },
        Expr::Div(left, right) => match (left.as_ref(), right.as_ref()) {
            (Expr::Num(a), Expr::Num(b)) if !zero(*b) => (Expr::Num(a / b), true),
            (_, Expr::Num(value)) if one(*value) => (*left, true),
            (Expr::Num(value), _) if zero(*value) && !matches!(right.as_ref(), Expr::Num(n) if zero(*n)) => {
                (Expr::Num(0.0), true)
            }
            _ => (Expr::Div(left, right), false),
        },
        Expr::Pow(base, exp) => match (base.as_ref(), exp.as_ref()) {
            (_, Expr::Num(value)) if one(*value) => (*base, true),
            (Expr::Num(base), _) if one(*base) => (Expr::Num(1.0), true),
            (_, Expr::Num(value)) if zero(*value) && !matches!(base.as_ref(), Expr::Num(n) if zero(*n)) => {
                (Expr::Num(1.0), true)
            }
            (Expr::Num(a), Expr::Num(b)) => {
                let value = a.powf(*b);
                if value.is_finite() {
                    (Expr::Num(value), true)
                } else {
                    (Expr::Pow(base, exp), false)
                }
            }
            _ => (Expr::Pow(base, exp), false),
        },
        other => (other, false),
    }
}

fn expr_to_source(expr: &Expr) -> String {
    match expr {
        Expr::Num(value) => number_source(*value),
        Expr::Var => "x".into(),
        Expr::Neg(inner) => format!("-({})", expr_to_source(inner)),
        Expr::Add(left, right) => format!("({}+{})", expr_to_source(left), expr_to_source(right)),
        Expr::Sub(left, right) => format!("({}-{})", expr_to_source(left), expr_to_source(right)),
        Expr::Mul(left, right) => format!("({}*{})", expr_to_source(left), expr_to_source(right)),
        Expr::Div(left, right) => format!("({}/{})", expr_to_source(left), expr_to_source(right)),
        Expr::Pow(left, right) => format!("({}^({}))", expr_to_source(left), expr_to_source(right)),
        Expr::Call(func, arg) => format!("{}({})", func.source_name(), expr_to_source(arg)),
        Expr::Index(name) => name.clone(),
        Expr::Sum { var, lower, upper, body } => format!(
            "sum_{{{var}={}}}^{{{}}} ({})",
            expr_to_source(lower),
            expr_to_source(upper),
            expr_to_source(body)
        ),
        Expr::Prod { var, lower, upper, body } => format!(
            "prod_{{{var}={}}}^{{{}}} ({})",
            expr_to_source(lower),
            expr_to_source(upper),
            expr_to_source(body)
        ),
        Expr::Limit { var, target, body } => format!(
            "lim_{{{var}->{}}} ({})",
            expr_to_source(target),
            expr_to_source(body)
        ),
    }
}

fn expr_to_plain(expr: &Expr) -> String {
    match expr {
        Expr::Num(value) => number_source(*value),
        Expr::Var => "x".into(),
        Expr::Neg(inner) => format!("-{}", plain_atom(inner)),
        Expr::Add(left, right) => format!("{} + {}", expr_to_plain(left), expr_to_plain(right)),
        Expr::Sub(left, right) => format!("{} - {}", expr_to_plain(left), plain_atom(right)),
        Expr::Mul(left, right) => format!("{}*{}", plain_atom(left), plain_atom(right)),
        Expr::Div(left, right) => format!("{}/{}", plain_atom(left), plain_atom(right)),
        Expr::Pow(left, right) => format!("{}^{}", plain_atom(left), expr_to_plain(right)),
        Expr::Call(func, arg) => format!("{}({})", func.source_name(), expr_to_plain(arg)),
        Expr::Index(name) => name.clone(),
        Expr::Sum { var, lower, upper, body } => format!(
            "sum_{{{var}={}}}^{{{}}} {}",
            expr_to_plain(lower),
            expr_to_plain(upper),
            expr_to_plain(body)
        ),
        Expr::Prod { var, lower, upper, body } => format!(
            "prod_{{{var}={}}}^{{{}}} {}",
            expr_to_plain(lower),
            expr_to_plain(upper),
            expr_to_plain(body)
        ),
        Expr::Limit { var, target, body } => {
            format!("lim_{{{var}->{}}} {}", expr_to_plain(target), expr_to_plain(body))
        }
    }
}

fn plain_atom(expr: &Expr) -> String {
    match expr {
        Expr::Num(_) | Expr::Var | Expr::Index(_) | Expr::Call(_, _) | Expr::Neg(_) => expr_to_plain(expr),
        _ => format!("({})", expr_to_plain(expr)),
    }
}

fn number_source(value: f64) -> String {
    if !value.is_finite() {
        return "0".into();
    }
    if (value - PI).abs() < 1e-8 {
        return "pi".into();
    }
    if (value + PI).abs() < 1e-8 {
        return "-(pi)".into();
    }
    if (value - E).abs() < 1e-8 {
        return "e".into();
    }
    if (value + E).abs() < 1e-8 {
        return "-(e)".into();
    }
    if value < 0.0 {
        format!("-({})", format_number(-value))
    } else {
        format_number(value)
    }
}

fn wolfram_normalize(input: &str) -> String {
    let pretty = replace_math_symbols(input.trim());
    let mut text = strip_leading_commands(&pretty);
    let (core, bounds) = peel_from_to(&text);
    text = core.to_string();
    text = rewrite_function_powers(&text);
    text = rewrite_letter_runs(&text);
    text = rewrite_function_of(&text);
    text = rewrite_phrases(&text);
    text = ensure_dx(&text);
    if let Some((lower, upper)) = bounds {
        text = format!("{text} from {lower} to {upper}");
    }
    text
}

fn replace_math_symbols(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '∑' => out.push_str("sum"),
            '∏' => out.push_str("prod"),
            '∞' => out.push_str("inf"),
            '→' => out.push_str("->"),
            _ => out.push(ch),
        }
    }
    out
}

fn strip_leading_commands(input: &str) -> String {
    let mut text = input.trim().to_string();
    loop {
        let lower = text.to_ascii_lowercase();
        let prefixes = [
            "what is ",
            "calculate ",
            "compute ",
            "evaluate ",
            "simplify ",
            "solve ",
            "plot ",
            "graph ",
            "the ",
        ];
        let stripped = prefixes.iter().find_map(|prefix| {
            lower
                .strip_prefix(prefix)
                .map(|_| text[prefix.len()..].trim().to_string())
        });
        match stripped {
            Some(next) if next != text => text = next,
            _ => break,
        }
    }
    text
}

fn rewrite_function_powers(input: &str) -> String {
    let mut out = String::new();
    let mut index = 0usize;
    let mut depth = 0i32;
    while index < input.len() {
        let ch = input[index..].chars().next().unwrap();
        if depth == 0 {
            if let Some((func, exponent, arg, next)) = match_func_power(input, index) {
                out.push('(');
                out.push_str(&func);
                out.push('(');
                out.push_str(&arg);
                out.push_str("))^(");
                out.push_str(&exponent);
                out.push(')');
                index = next;
                continue;
            }
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        }
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

fn match_func_power(input: &str, start: usize) -> Option<(String, String, String, usize)> {
    let rest = input.get(start..)?;
    let lower = rest.to_ascii_lowercase();
    let prev_ok = start == 0 || !input.as_bytes()[start - 1].is_ascii_alphanumeric();
    if !prev_ok {
        return None;
    }
    let name = FUNC_NAMES
        .iter()
        .map(|(name, _)| *name)
        .chain(["exp"])
        .find(|name| lower.starts_with(name))?;
    let mut index = start + name.len();
    if input.as_bytes().get(index).is_some_and(|byte| byte.is_ascii_alphabetic()) {
        return None;
    }
    index = skip_ascii_ws(input, index);
    if input[index..].starts_with("**") {
        index += 2;
    } else if input[index..].starts_with('^') {
        index += 1;
    } else {
        return None;
    }
    index = skip_ascii_ws(input, index);
    let (exponent, mut index) = read_exponent(input, index)?;
    index = skip_ascii_ws(input, index);
    let (arg, index) = read_power_arg(input, index)?;
    Some((name.to_string(), exponent, arg, index))
}

fn read_exponent(input: &str, index: usize) -> Option<(String, usize)> {
    if input[index..].starts_with('(') {
        let (inside, rest) = take_balanced_paren(&input[index..])?;
        let next = index + (input[index..].len() - rest.len());
        return Some((inside.trim().to_string(), next));
    }
    let bytes = input.as_bytes();
    let mut end = index;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    (end > index).then(|| (input[index..end].to_string(), end))
}

fn read_power_arg(input: &str, index: usize) -> Option<(String, usize)> {
    if input[index..].starts_with('(') {
        let (inside, rest) = take_balanced_paren(&input[index..])?;
        let next = index + (input[index..].len() - rest.len());
        return Some((inside.trim().to_string(), next));
    }
    let bytes = input.as_bytes();
    let mut end = index;
    while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
        end += 1;
    }
    let letters = end;
    while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
        end += 1;
    }
    (end > letters).then(|| (input[index..end].to_string(), end))
}

fn rewrite_letter_runs(input: &str) -> String {
    let mut out = String::new();
    let mut index = 0usize;
    while index < input.len() {
        let ch = input[index..].chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            let start = index;
            let bytes = input.as_bytes();
            while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
                index += 1;
            }
            let run = &input[start..index];
            let prev_digit = start > 0 && input[..start].chars().next_back().is_some_and(|ch| ch.is_ascii_digit());
            if prev_digit && is_scientific_e(run, &input[index..]) {
                out.push_str(run);
            } else if let Some(rewritten) = split_known_run(run) {
                out.push_str(&rewritten);
            } else {
                out.push_str(run);
            }
        } else {
            out.push(ch);
            index += ch.len_utf8();
        }
    }
    out
}

fn is_scientific_e(run: &str, after: &str) -> bool {
    if !run.eq_ignore_ascii_case("e") {
        return false;
    }
    let bytes = after.as_bytes();
    if bytes.is_empty() {
        return true;
    }
    let mut index = 0usize;
    if bytes[0] == b'+' || bytes[0] == b'-' {
        index = 1;
    }
    bytes.get(index).is_some_and(|byte| byte.is_ascii_digit())
}

fn split_known_run(run: &str) -> Option<String> {
    let lower = run.to_ascii_lowercase();
    let tokens = tokenize_run(&lower)?;
    Some(render_tokens(&tokens))
}

fn tokenize_run(source: &str) -> Option<Vec<&str>> {
    if source.is_empty() {
        return Some(Vec::new());
    }
    for (word, _) in FUNC_NAMES {
        if let Some(rest) = source.strip_prefix(word) {
            if let Some(mut tokens) = tokenize_run(rest) {
                tokens.insert(0, word);
                return Some(tokens);
            }
        }
    }
    for word in ["exp", "pi", "e", "x"] {
        if let Some(rest) = source.strip_prefix(word) {
            if let Some(mut tokens) = tokenize_run(rest) {
                tokens.insert(0, word);
                return Some(tokens);
            }
        }
    }
    None
}

fn render_tokens(tokens: &[&str]) -> String {
    let mut parts = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if is_func_token(tokens[index]) {
            let start = index;
            while index < tokens.len() && is_func_token(tokens[index]) {
                index += 1;
            }
            if index < tokens.len() && tokens[index] == "x" {
                let mut expr = "x".to_string();
                for func in tokens[start..index].iter().rev() {
                    expr = apply_func_token(func, &expr);
                }
                index += 1;
                parts.push(expr);
            } else {
                for func in &tokens[start..index] {
                    parts.push((*func).to_string());
                }
            }
        } else {
            parts.push(tokens[index].to_string());
            index += 1;
        }
    }
    parts.join("*")
}

fn is_func_token(token: &str) -> bool {
    token == "exp" || FUNC_NAMES.iter().any(|(name, _)| *name == token)
}

fn apply_func_token(func: &str, arg: &str) -> String {
    if func == "exp" {
        format!("(e^({arg}))")
    } else {
        format!("{func}({arg})")
    }
}

fn rewrite_function_of(input: &str) -> String {
    let funcs = ["sqrt", "log10", "exp", "sin", "cos", "tan", "abs", "log", "ln"];
    let lower = input.to_ascii_lowercase();
    let mut out = String::new();
    let mut index = 0usize;
    let mut depth = 0i32;
    while index < input.len() {
        let ch = input[index..].chars().next().unwrap();
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        }
        if depth == 0 {
            if let Some(func) = funcs.iter().copied().find(|func| lower[index..].starts_with(func)) {
                let after = index + func.len();
                let prev_ok = index == 0 || !input.as_bytes()[index - 1].is_ascii_alphabetic();
                let next_ok = after >= input.len() || !input.as_bytes()[after].is_ascii_alphabetic();
                if prev_ok && next_ok && lower[after..].starts_with(" of") {
                    let mut of_end = after + " of".len();
                    if of_end == input.len() || input[of_end..].starts_with(|ch: char| ch.is_whitespace()) {
                        out.push_str(&input[index..after]);
                        out.push(' ');
                        of_end = skip_ascii_ws(input, of_end);
                        index = of_end;
                        continue;
                    }
                }
            }
        }
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

fn rewrite_phrases(input: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let mut depth = 0i32;
    let mut index = 0usize;
    while index < input.len() {
        let ch = input[index..].chars().next().unwrap();
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        } else if depth == 0 {
            if let Some(body) = match_phrase(input, &lower, index) {
                return format!("{}{body}", &input[..index]);
            }
        }
        index += ch.len_utf8();
    }
    input.to_string()
}

fn match_phrase(input: &str, lower: &str, index: usize) -> Option<String> {
    let prev_ok = index == 0 || !input.as_bytes()[index - 1].is_ascii_alphabetic();
    if !prev_ok {
        return None;
    }
    if lower[index..].starts_with("d/dx of") {
        let end = index + "d/dx of".len();
        if phrase_boundary(input, end) {
            let rest = rewrite_phrases(input[end..].trim());
            return Some(format!("d/dx {rest}"));
        }
    }
    let derivatives = [
        ("second derivative of", 2u32),
        ("third derivative of", 3),
        ("2nd derivative of", 2),
        ("3rd derivative of", 3),
        ("first derivative of", 1),
        ("derivative of", 1),
        ("deriv of", 1),
        ("differentiate", 1),
    ];
    for (phrase, order) in derivatives {
        if lower[index..].starts_with(phrase) {
            let end = index + phrase.len();
            if phrase_boundary(input, end) {
                let rest = rewrite_phrases(input[end..].trim());
                return Some(apply_derivative(order, &rest));
            }
        }
    }
    let integrals = ["integral of", "integrate", "integral"];
    for phrase in integrals {
        if lower[index..].starts_with(phrase) {
            let end = index + phrase.len();
            if phrase_boundary(input, end) {
                let rest = rewrite_phrases(input[end..].trim());
                let integrand = strip_trailing_dx(&rest);
                return Some(format!("int ({integrand}) dx"));
            }
        }
    }
    None
}

fn phrase_boundary(input: &str, end: usize) -> bool {
    input[end..]
        .chars()
        .next()
        .is_none_or(|ch| !ch.is_ascii_alphabetic())
}

fn apply_derivative(order: u32, rest: &str) -> String {
    if let Some((inner, body)) = strip_derivative_prefix(rest) {
        let total = (order + inner).min(6);
        let body = body.trim();
        if total <= 1 {
            format!("d/dx {body}")
        } else {
            format!("d^{total}/dx^{total} {body}")
        }
    } else if order <= 1 {
        format!("d/dx ({rest})")
    } else {
        format!("d^{order}/dx^{order} ({rest})")
    }
}

fn strip_trailing_dx(input: &str) -> String {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.len() >= 2 && lower.ends_with("dx") {
        let cut = trimmed.len() - 2;
        let before = trimmed[..cut].chars().next_back();
        if before.is_none_or(|ch| !ch.is_ascii_alphabetic()) {
            let body = trimmed[..cut].trim();
            if !body.is_empty() {
                return body.to_string();
            }
        }
    }
    trimmed.to_string()
}

fn ensure_dx(input: &str) -> String {
    if has_integral_keyword(input) && !has_dx_token(input) {
        format!("{input} dx")
    } else {
        input.to_string()
    }
}

fn has_integral_keyword(input: &str) -> bool {
    if input.contains('∫') {
        return true;
    }
    let lower = input.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut index = 0usize;
    while index < lower.len() {
        let key_len = if lower[index..].starts_with("integral") {
            Some(8)
        } else if lower[index..].starts_with("int") {
            Some(3)
        } else {
            None
        };
        if let Some(key_len) = key_len {
            let prev_ok = index == 0 || !bytes[index - 1].is_ascii_alphabetic();
            let next = index + key_len;
            let next_ok = next >= bytes.len() || !bytes[next].is_ascii_alphabetic();
            if prev_ok && next_ok {
                return true;
            }
        }
        index += lower[index..].chars().next().unwrap().len_utf8();
    }
    false
}

fn has_dx_token(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut depth = 0i32;
    let mut index = 0usize;
    while index < lower.len() {
        let ch = lower[index..].chars().next().unwrap();
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            'd' if depth == 0 && lower[index..].starts_with("dx") => {
                let prev_ok = index == 0 || !bytes[index - 1].is_ascii_alphabetic();
                let next = index + 2;
                let next_ok = next >= bytes.len() || !bytes[next].is_ascii_alphabetic();
                if prev_ok && next_ok {
                    return true;
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    false
}

pub fn looks_like_math(input: &str) -> bool {
    let trimmed = wolfram_normalize(input.trim());
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('=') {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    if matches!(lower.as_str(), "pi" | "e" | "x" | "π") {
        return true;
    }
    let compact = lower.split_whitespace().collect::<String>();
    if compact.starts_with("y=") {
        return true;
    }
    if lower.contains('∫')
        || compact.starts_with("d/dx")
        || compact.starts_with("int(")
        || compact.starts_with("int ")
        || compact.starts_with("integral")
    {
        return true;
    }
    for name in FUNC_NAMES.iter().map(|(name, _)| *name).chain(["sum", "prod", "product", "lim", "limit"]) {
        if let Some(rest) = lower.strip_prefix(name) {
            if rest.is_empty() || rest == "x" || rest.starts_with(['(', ' ', '_']) {
                return true;
            }
        }
    }
    let allowed = trimmed.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || ch.is_whitespace()
            || matches!(
                ch,
                '+' | '-' | '*' | '/' | '^' | '(' | ')' | '.' | '=' | ';' | '×' | '÷' | 'π' | '!' | '_' | '{' | '}' | '>'
            )
    });
    if !allowed {
        return false;
    }
    trimmed
        .chars()
        .any(|ch| matches!(ch, '+' | '-' | '*' | '/' | '^' | '(' | ')' | '=' | '×' | '÷' | 'π') || ch.is_ascii_digit())
}

pub fn parse_expression(input: &str) -> Result<Expr, String> {
    let normalized = wolfram_normalize(input);
    let mut parser = Parser { s: &normalized, i: 0, bounds: Vec::new() };
    let expr = parser.parse_expr()?;
    parser.skip_ws();
    if parser.i != parser.s.len() {
        return Err("Unexpected input".into());
    }
    Ok(expr)
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
    bounds: Vec<String>,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn eat_char(&mut self, want: char) -> bool {
        if self.peek_char() == Some(want) {
            self.i += want.len_utf8();
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) -> bool {
        let start = self.i;
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.i += ch.len_utf8();
            } else {
                break;
            }
        }
        self.i > start
    }

    fn peek_ident(&self) -> bool {
        self.peek_char().is_some_and(|ch| ch.is_ascii_alphabetic())
    }

    fn peek_number(&self) -> bool {
        let bytes = self.rest().as_bytes();
        match bytes.first() {
            Some(b) if b.is_ascii_digit() => true,
            Some(b'.') => bytes.get(1).is_some_and(|d| d.is_ascii_digit()),
            _ => false,
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            if self.eat_char('+') {
                left = Expr::Add(Box::new(left), Box::new(self.parse_term()?));
            } else if self.eat_char('-') {
                left = Expr::Sub(Box::new(left), Box::new(self.parse_term()?));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;
        loop {
            let mark = self.i;
            self.skip_ws();
            if self.eat_char('×') {
                left = Expr::Mul(Box::new(left), Box::new(self.parse_factor()?));
                continue;
            }
            if self.eat_char('÷') {
                left = Expr::Div(Box::new(left), Box::new(self.parse_factor()?));
                continue;
            }
            if self.eat_char('*') {
                left = Expr::Mul(Box::new(left), Box::new(self.parse_factor()?));
                continue;
            }
            if self.eat_char('/') {
                left = Expr::Div(Box::new(left), Box::new(self.parse_factor()?));
                continue;
            }
            if self.starts_implicit() {
                left = Expr::Mul(Box::new(left), Box::new(self.parse_factor()?));
                if self.i == mark {
                    return Err("Could not parse that expression".into());
                }
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn starts_implicit(&self) -> bool {
        match self.peek_char() {
            Some('(' | 'π') => true,
            Some(ch) if ch.is_ascii_digit() || ch.is_ascii_alphabetic() => true,
            Some('.') => self.peek_number(),
            _ => false,
        }
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat_char('+') {
            return self.parse_factor();
        }
        if self.eat_char('-') {
            return Ok(Expr::Neg(Box::new(self.parse_factor()?)));
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_primary()?;
        self.skip_ws();
        if self.eat_char('^') || self.eat_star_star() {
            let exponent = self.parse_factor()?;
            return Ok(Expr::Pow(Box::new(base), Box::new(exponent)));
        }
        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let expr = self.parse_atom()?;
        self.wrap_factorial(expr)
    }

    fn wrap_factorial(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            let save = self.i;
            self.skip_ws();
            if self.eat_char('!') {
                expr = Expr::Call(Func::Fact, Box::new(expr));
            } else {
                self.i = save;
                break;
            }
        }
        Ok(expr)
    }

    fn eat_star_star(&mut self) -> bool {
        let bytes = self.rest().as_bytes();
        if bytes.len() >= 2 && bytes[0] == b'*' && bytes[1] == b'*' {
            self.i += 2;
            true
        } else {
            false
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat_char('(') {
            let inner = self.parse_expr()?;
            self.skip_ws();
            if !self.eat_char(')') {
                return Err("Expected ')'".into());
            }
            return Ok(inner);
        }
        if self.eat_char('π') {
            return Ok(Expr::Num(PI));
        }
        if self.peek_number() {
            return self.parse_number();
        }
        if self.peek_ident() {
            let name = self.consume_name();
            let lower = name.to_ascii_lowercase();
            if matches!(lower.as_str(), "sum" | "prod" | "product") {
                return self.parse_fold(lower == "sum");
            }
            if matches!(lower.as_str(), "lim" | "limit") {
                return self.parse_limit();
            }
            if self.bounds.iter().any(|bound| bound == &lower) {
                return Ok(Expr::Index(lower));
            }
            if lower == "log" && self.consume_log10() {
                return self.parse_applied_call(Func::Log, &name);
            }
            if lower == "log" && self.consume_log2() {
                return self.parse_applied_call(Func::Log2, &name);
            }
            if lower == "exp" {
                let arg = self.parse_applied_argument(&name)?;
                return Ok(Expr::Pow(Box::new(Expr::Num(E)), Box::new(arg)));
            }
            if let Some(func) = Func::from_name(&lower) {
                return self.parse_applied_call(func, &name);
            }
            return match lower.as_str() {
                "pi" => Ok(Expr::Num(PI)),
                "e" => Ok(Expr::Num(E)),
                "i" => Ok(Expr::Index("i".into())),
                "inf" | "infinity" | "oo" => Ok(Expr::Num(f64::INFINITY)),
                "x" => Ok(Expr::Var),
                _ => Err(format!("Unknown name '{name}'")),
            };
        }
        Err("Expected an expression".into())
    }

    fn parse_fold_limit(&mut self) -> Result<(String, Expr), String> {
        self.skip_ws();
        if self.peek_ident() {
            let mark = self.i;
            let name = self.parse_ident().to_ascii_lowercase();
            self.skip_ws();
            if self.eat_char('=') {
                return Ok((name, self.parse_expr()?));
            }
            self.i = mark;
        }
        Ok(("n".into(), self.parse_expr()?))
    }

    fn parse_fold(&mut self, sum: bool) -> Result<Expr, String> {
        let kind = if sum { "sum" } else { "prod" };
        self.skip_ws();
        if !self.eat_char('_') {
            return Err(format!("Write {kind}_{{1}}^{{10}} n"));
        }
        self.skip_ws();
        if !self.eat_char('{') {
            return Err(format!("Write {kind}_{{1}}^{{10}} n"));
        }
        let (var, lower) = self.parse_fold_limit()?;
        self.skip_ws();
        if !self.eat_char('}') {
            return Err("Expected '}'".into());
        }
        self.skip_ws();
        if !self.eat_char('^') {
            return Err("Expected an upper limit".into());
        }
        self.skip_ws();
        let upper = if self.eat_char('{') {
            let upper = self.parse_expr()?;
            self.skip_ws();
            if !self.eat_char('}') {
                return Err("Expected '}'".into());
            }
            upper
        } else {
            self.parse_power()?
        };
        self.bounds.push(var.clone());
        let body = self.parse_term();
        self.bounds.pop();
        let body = body?;
        if sum {
            Ok(Expr::Sum {
                var,
                lower: Box::new(lower),
                upper: Box::new(upper),
                body: Box::new(body),
            })
        } else {
            Ok(Expr::Prod {
                var,
                lower: Box::new(lower),
                upper: Box::new(upper),
                body: Box::new(body),
            })
        }
    }

    fn parse_limit(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if !self.eat_char('_') {
            return Err("Write a limit as lim_{x->0} f(x)".into());
        }
        self.skip_ws();
        if !self.eat_char('{') {
            return Err("Write a limit as lim_{x->0} f(x)".into());
        }
        self.skip_ws();
        if !self.peek_ident() {
            return Err("Expected a variable name".into());
        }
        let var = self.parse_ident().to_ascii_lowercase();
        self.skip_ws();
        if !self.eat_arrow() {
            return Err("Expected '->' in the limit".into());
        }
        let target = self.parse_expr()?;
        self.skip_ws();
        if !self.eat_char('}') {
            return Err("Expected '}'".into());
        }
        self.bounds.push(var.clone());
        let body = self.parse_term();
        self.bounds.pop();
        Ok(Expr::Limit {
            var,
            target: Box::new(target),
            body: Box::new(body?),
        })
    }

    fn eat_arrow(&mut self) -> bool {
        self.skip_ws();
        let rest = self.rest();
        if rest.starts_with("->") {
            self.i += 2;
            true
        } else if let Some(rest) = rest.strip_prefix('→') {
            self.i += self.rest().len() - rest.len();
            true
        } else {
            false
        }
    }

    fn consume_log10(&mut self) -> bool {
        self.consume_log_digits("10")
    }

    fn consume_log2(&mut self) -> bool {
        self.consume_log_digits("2")
    }

    fn consume_log_digits(&mut self, digits: &str) -> bool {
        let rest = self.rest();
        if rest.starts_with(digits)
            && rest[digits.len()..]
                .chars()
                .next()
                .is_none_or(|ch| !ch.is_ascii_digit())
        {
            self.i += digits.len();
            return true;
        }
        false
    }

    fn parse_applied_call(&mut self, func: Func, name: &str) -> Result<Expr, String> {
        Ok(Expr::Call(func, Box::new(self.parse_applied_argument(name)?)))
    }

    fn parse_applied_argument(&mut self, name: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat_char('(') {
            let arg = self.parse_expr()?;
            self.skip_ws();
            if !self.eat_char(')') {
                return Err("Expected ')'".into());
            }
            return Ok(arg);
        }
        let juxtapose = self.peek_ident()
            || self.peek_char() == Some('π')
            || self.peek_char() == Some('(')
            || self.peek_number();
        if juxtapose {
            return self.parse_term();
        }
        Err(format!("Expected an argument for {name}"))
    }

    fn parse_ident(&mut self) -> String {
        let source = self.rest();
        let mut end = 0;
        for (index, ch) in source.char_indices() {
            if ch.is_ascii_alphabetic() {
                end = index + ch.len_utf8();
            } else {
                break;
            }
        }
        self.i += end;
        source[..end].to_string()
    }

    fn consume_name(&mut self) -> String {
        let source = self.rest();
        let mut end = 0;
        for (index, ch) in source.char_indices() {
            if ch.is_ascii_alphabetic() {
                end = index + ch.len_utf8();
            } else {
                break;
            }
        }
        let word = source[..end].to_string();
        let cut = name_cut(&word.to_ascii_lowercase(), &self.bounds);
        self.i += cut;
        word[..cut].to_string()
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        let source = self.rest();
        let bytes = source.as_bytes();
        let mut i = 0usize;
        let mut saw_digit = false;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            saw_digit = true;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
                saw_digit = true;
            }
        }
        if !saw_digit {
            return Err("Bad number".into());
        }
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            let mut j = i + 1;
            if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
            }
            let exp_digits = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > exp_digits {
                i = j;
            }
        }
        let text = &source[..i];
        self.i += i;
        let number = text
            .parse::<f64>()
            .map_err(|_| format!("Bad number '{text}'"))?;
        Ok(Expr::Num(number))
    }
}

pub fn eval(expr: &Expr, x: Option<f64>) -> Result<f64, String> {
    eval_in(expr, x, &[])
}

fn eval_in(expr: &Expr, x: Option<f64>, locals: &[(&str, f64)]) -> Result<f64, String> {
    let value = match expr {
        Expr::Num(number) => *number,
        Expr::Var => x.ok_or("This expression needs x")?,
        Expr::Index(name) => {
            return locals
                .iter()
                .rev()
                .find(|(key, _)| key == name)
                .map(|(_, value)| *value)
                .ok_or_else(|| format!("Unknown name '{name}'"));
        }
        Expr::Neg(inner) => -eval_in(inner, x, locals)?,
        Expr::Add(a, b) => eval_in(a, x, locals)? + eval_in(b, x, locals)?,
        Expr::Sub(a, b) => eval_in(a, x, locals)? - eval_in(b, x, locals)?,
        Expr::Mul(a, b) => eval_in(a, x, locals)? * eval_in(b, x, locals)?,
        Expr::Div(a, b) => eval_in(a, x, locals)? / eval_in(b, x, locals)?,
        Expr::Pow(a, b) => eval_in(a, x, locals)?.powf(eval_in(b, x, locals)?),
        Expr::Call(func, arg) => func.apply(eval_in(arg, x, locals)?),
        Expr::Sum { var, lower, upper, body } => {
            return fold_range(var, lower, upper, body, x, locals, true);
        }
        Expr::Prod { var, lower, upper, body } => {
            return fold_range(var, lower, upper, body, x, locals, false);
        }
        Expr::Limit { var, target, body } => return eval_limit(var, target, body, x, locals),
    };
    Ok(value)
}

fn fold_range(
    var: &str,
    lower: &Expr,
    upper: &Expr,
    body: &Expr,
    x: Option<f64>,
    locals: &[(&str, f64)],
    sum: bool,
) -> Result<f64, String> {
    let low = as_bound(eval_in(lower, x, locals)?)
        .ok_or("Sum and product limits should be integers")?;
    let high = as_bound(eval_in(upper, x, locals)?)
        .ok_or("Sum and product limits should be integers")?;
    if high < low {
        return Ok(if sum { 0.0 } else { 1.0 });
    }
    let count = (high as i128 - low as i128 + 1).unsigned_abs();
    if count > 20_000 {
        return Err("That sum is too long".into());
    }
    let mut acc = if sum { 0.0 } else { 1.0 };
    for k in low..=high {
        let mut next = locals.to_vec();
        next.push((var, k as f64));
        let value = eval_in(body, x, &next)?;
        if sum {
            acc += value;
        } else {
            acc *= value;
        }
    }
    Ok(acc)
}

fn as_bound(value: f64) -> Option<i64> {
    if !value.is_finite() {
        return None;
    }
    let rounded = value.round();
    ((value - rounded).abs() <= 1e-8 && rounded.abs() <= 1e12).then_some(rounded as i64)
}

fn eval_limit(
    var: &str,
    target: &Expr,
    body: &Expr,
    x: Option<f64>,
    locals: &[(&str, f64)],
) -> Result<f64, String> {
    let point = eval_in(target, x, locals)?;
    let probe = |at: f64| {
        let mut next = locals.to_vec();
        next.push((var, at));
        eval_in(body, x, &next)
    };
    if point.is_nan() {
        return Err("That limit is undefined".into());
    }
    if point.is_infinite() {
        let sign = if point.is_sign_positive() { 1.0 } else { -1.0 };
        let mut last: Option<f64> = None;
        for mag in [1e3, 1e4, 1e5, 1e6] {
            let value = probe(sign * mag)?;
            if let Some(prev) = last {
                if value.is_finite() && (value - prev).abs() <= 1e-3 * (1.0 + value.abs()) {
                    return Ok(value);
                }
            }
            if value.is_finite() {
                last = Some(value);
            }
        }
        return last.ok_or("That limit does not settle".to_string());
    }
    let step = 1e-5 * (1.0 + point.abs());
    let right = probe(point + step);
    let left = probe(point - step);
    match (right, left) {
        (Ok(a), Ok(b))
            if a.is_finite()
                && b.is_finite()
                && (a - b).abs() <= 1e-3 * (1.0 + a.abs().max(b.abs())) =>
        {
            Ok((a + b) / 2.0)
        }
        (Ok(a), _) if a.is_finite() => Ok(a),
        (_, Ok(b)) if b.is_finite() => Ok(b),
        _ => Err("That limit is undefined".into()),
    }
}

fn factorial(value: f64) -> f64 {
    if !value.is_finite() {
        return f64::NAN;
    }
    if (value - value.round()).abs() < 1e-8 {
        let n = value.round();
        if n < 0.0 || n > 170.0 {
            return f64::NAN;
        }
        let mut acc = 1.0;
        for k in 2..=(n as i64) {
            acc *= k as f64;
        }
        return acc;
    }
    gamma(value + 1.0)
}

fn gamma(z: f64) -> f64 {
    if !z.is_finite() {
        return f64::NAN;
    }
    if z < 0.5 {
        return PI / ((PI * z).sin() * gamma(1.0 - z));
    }
    let z = z - 1.0;
    let coeffs = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    let mut acc = coeffs[0];
    for (index, coeff) in coeffs.iter().enumerate().skip(1) {
        acc += coeff / (z + index as f64);
    }
    let t = z + 7.5;
    (2.0 * PI).sqrt() * t.powf(z + 0.5) * (-t).exp() * acc
}

fn digamma(mut x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x < 0.0 {
        return digamma(1.0 - x) - PI / (PI * x).tan();
    }
    let mut result = 0.0;
    while x < 8.0 {
        result -= 1.0 / x;
        x += 1.0;
    }
    let inv = 1.0 / x;
    let inv2 = inv * inv;
    result + x.ln() - 0.5 * inv - inv2 * (1.0 / 12.0 - inv2 * (1.0 / 120.0 - inv2 / 252.0))
}

fn trigamma(mut x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x < 0.0 {
        let s = (PI * x).sin();
        return -trigamma(1.0 - x) + (PI / s) * (PI / s);
    }
    let mut result = 0.0;
    while x < 8.0 {
        result += 1.0 / (x * x);
        x += 1.0;
    }
    let inv = 1.0 / x;
    let inv2 = inv * inv;
    result + inv + 0.5 * inv2 + inv2 * inv * (1.0 / 6.0 - inv2 * (1.0 / 30.0 - inv2 / 42.0))
}

pub fn format_number(value: f64) -> String {
    if value.is_infinite() {
        return if value.is_sign_positive() { "∞".into() } else { "-∞".into() };
    }
    if !value.is_finite() {
        return "undefined".into();
    }
    if value.abs() < 1e-12 {
        return "0".into();
    }
    let rounded = value.round();
    if (value - rounded).abs() < 1e-9 * (1.0 + value.abs()) && rounded.abs() < 1e15 {
        return format!("{}", rounded as i64);
    }
    let abs = value.abs();
    if abs >= 1e12 || abs < 1e-6 {
        return format!("{value:.6e}");
    }
    let text = format!("{value:.12}");
    trim_zeros(&text)
}

fn trim_zeros(text: &str) -> String {
    if !text.contains('.') {
        return text.to_string();
    }
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".into()
    } else {
        trimmed.to_string()
    }
}

pub fn number_latex(value: f64) -> String {
    if value.is_infinite() {
        return if value.is_sign_positive() { "\\infty".into() } else { "-\\infty".into() };
    }
    if !value.is_finite() {
        return "\\text{undefined}".into();
    }
    if value.abs() < 1e-12 {
        return "0".into();
    }
    if (value - E).abs() <= 1e-8 {
        return "e".into();
    }
    if (value + E).abs() <= 1e-8 {
        return "-e".into();
    }
    if let Some(pretty) = pi_latex(value) {
        return pretty;
    }
    let rounded = value.round();
    if (value - rounded).abs() < 1e-9 * (1.0 + value.abs()) && rounded.abs() < 1e15 {
        return format!("{}", rounded as i64);
    }
    if let Some((numer, denom)) = rational_pair(value, 12) {
        if denom != 1 {
            return fraction_latex(numer, denom);
        }
    }
    decimal_latex(value)
}

fn decimal_latex(value: f64) -> String {
    let text = format_number(value);
    if let Some((mantissa, exp)) = text.split_once(['e', 'E']) {
        let exp = exp.trim_start_matches('+');
        return format!("{mantissa}\\times 10^{{{exp}}}");
    }
    text
}

fn pi_latex(value: f64) -> Option<String> {
    let (numer, denom) = rational_pair(value / PI, 12)?;
    if numer == 0 {
        return None;
    }
    let sign = if numer < 0 { "-" } else { "" };
    let numer = numer.abs();
    Some(match (numer, denom) {
        (1, 1) => format!("{sign}\\pi"),
        (_, 1) => format!("{sign}{numer}\\pi"),
        (1, _) => format!("{sign}\\frac{{\\pi}}{{{denom}}}"),
        _ => format!("{sign}\\frac{{{numer}\\pi}}{{{denom}}}"),
    })
}

fn fraction_latex(numer: i32, denom: i32) -> String {
    let sign = if numer < 0 { "-" } else { "" };
    format!("{sign}\\frac{{{}}}{{{}}}", numer.abs(), denom)
}

fn rational_pair(value: f64, max_den: i32) -> Option<(i32, i32)> {
    if !value.is_finite() {
        return None;
    }
    for denom in 1..=max_den {
        let numer = (value * f64::from(denom)).round();
        if !numer.is_finite() || numer.abs() > 240.0 {
            continue;
        }
        let approx = numer / f64::from(denom);
        if (value - approx).abs() <= 1e-8 * (1.0 + value.abs()) {
            return Some((numer as i32, denom));
        }
    }
    None
}

fn call_latex(func: Func, arg: &Expr) -> String {
    let inner = latex_prec(arg, 0);
    match func {
        Func::Sqrt => format!("\\sqrt{{{inner}}}"),
        Func::Abs => format!("\\left|{inner}\\right|"),
        Func::Floor => format!("\\left\\lfloor {inner}\\right\\rfloor"),
        Func::Ceil => format!("\\left\\lceil {inner}\\right\\rceil"),
        Func::Fact => {
            let atom = match arg {
                Expr::Num(_) | Expr::Var | Expr::Index(_) | Expr::Call(_, _) => latex_prec(arg, 6),
                _ => format!("\\left({inner}\\right)"),
            };
            format!("{atom}!")
        }
        Func::Gamma => format!("\\Gamma\\left({inner}\\right)"),
        Func::Digamma => format!("\\psi\\left({inner}\\right)"),
        Func::Trigamma => format!("\\psi_{{1}}\\left({inner}\\right)"),
        Func::Sign => format!("\\operatorname{{sign}}\\left({inner}\\right)"),
        Func::Round => format!("\\operatorname{{round}}\\left({inner}\\right)"),
        Func::Sec => format!("\\sec\\left({inner}\\right)"),
        Func::Csc => format!("\\csc\\left({inner}\\right)"),
        Func::Cot => format!("\\cot\\left({inner}\\right)"),
        Func::Sin => format!("\\sin\\left({inner}\\right)"),
        Func::Cos => format!("\\cos\\left({inner}\\right)"),
        Func::Tan => format!("\\tan\\left({inner}\\right)"),
        Func::Asin => format!("\\arcsin\\left({inner}\\right)"),
        Func::Acos => format!("\\arccos\\left({inner}\\right)"),
        Func::Atan => format!("\\arctan\\left({inner}\\right)"),
        Func::Sinh => format!("\\sinh\\left({inner}\\right)"),
        Func::Cosh => format!("\\cosh\\left({inner}\\right)"),
        Func::Tanh => format!("\\tanh\\left({inner}\\right)"),
        Func::Ln => format!("\\log\\left({inner}\\right)"),
        Func::Log => format!("\\log_{{10}}\\left({inner}\\right)"),
        Func::Log2 => format!("\\log_{{2}}\\left({inner}\\right)"),
        other => format!("\\operatorname{{{}}}\\left({inner}\\right)", other.source_name()),
    }
}

fn latex_expr(expr: &Expr) -> String {
    latex_prec(expr, 0)
}

fn latex_prec(expr: &Expr, min_prec: u8) -> String {
    let (prec, body) = match expr {
        Expr::Num(number) => (6, number_latex(*number)),
        Expr::Var => (6, "x".into()),
        Expr::Neg(inner) => (4, format!("-{}", latex_prec(inner, 4))),
        Expr::Add(left, right) => (
            1,
            format!("{} + {}", latex_prec(left, 1), latex_prec(right, 2)),
        ),
        Expr::Sub(left, right) => (
            1,
            format!("{} - {}", latex_prec(left, 1), latex_prec(right, 2)),
        ),
        Expr::Mul(left, right) => (3, mul_latex(left, right)),
        Expr::Div(left, right) => (
            6,
            format!(
                "\\frac{{{}}}{{{}}}",
                latex_prec(left, 0),
                latex_prec(right, 0)
            ),
        ),
        Expr::Pow(base, exp) => (
            5,
            format!(
                "{}^{{{}}}",
                latex_prec(base, 6),
                latex_prec(exp, 0)
            ),
        ),
        Expr::Index(name) => (6, name.clone()),
        Expr::Call(func, arg) => (6, call_latex(*func, arg)),
        Expr::Sum { var, lower, upper, body } => (
            2,
            format!(
                "\\sum_{{{var}={}}}^{{{}}} {}",
                latex_prec(lower, 0),
                latex_prec(upper, 0),
                latex_prec(body, 3)
            ),
        ),
        Expr::Prod { var, lower, upper, body } => (
            2,
            format!(
                "\\prod_{{{var}={}}}^{{{}}} {}",
                latex_prec(lower, 0),
                latex_prec(upper, 0),
                latex_prec(body, 3)
            ),
        ),
        Expr::Limit { var, target, body } => (
            2,
            format!(
                "\\lim_{{{var} \\to {}}} {}",
                latex_prec(target, 0),
                latex_prec(body, 3)
            ),
        ),
    };
    if prec < min_prec {
        format!("\\left({body}\\right)")
    } else {
        body
    }
}

fn mul_latex(left: &Expr, right: &Expr) -> String {
    let lhs = latex_prec(left, 3);
    let rhs = latex_prec(right, 4);
    if juxtapose(left, right) {
        format!("{lhs}{rhs}")
    } else {
        format!("{lhs} \\cdot {rhs}")
    }
}

fn juxtapose(left: &Expr, right: &Expr) -> bool {
    let Expr::Num(number) = left else {
        return false;
    };
    if !number.is_finite() || number.is_sign_negative() || is_special_constant(*number) {
        return false;
    }
    !matches!(right, Expr::Num(value) if !is_special_constant(*value))
}

fn is_special_constant(value: f64) -> bool {
    (value - PI).abs() <= 1e-8 || (value - E).abs() <= 1e-8
}

enum Solved {
    All,
    None,
    Roots { values: Vec<f64>, bounded: bool },
}

enum PolySolve {
    All,
    None,
    Roots(Vec<f64>),
    Defer,
}

fn solve_real(expr: &Expr) -> Solved {
    if let Some(poly) = polynomial(expr) {
        match solve_poly(&poly) {
            PolySolve::All => return Solved::All,
            PolySolve::None => return Solved::None,
            PolySolve::Roots(values) => {
                return Solved::Roots {
                    values,
                    bounded: false,
                };
            }
            PolySolve::Defer => {}
        }
    }
    match numerical_roots(expr, -20.0, 20.0) {
        Numeric::All => Solved::All,
        Numeric::Roots(values) if values.is_empty() => Solved::None,
        Numeric::Roots(values) => Solved::Roots {
            values,
            bounded: true,
        },
    }
}

enum Numeric {
    All,
    Roots(Vec<f64>),
}

fn numerical_roots(expr: &Expr, lo: f64, hi: f64) -> Numeric {
    let steps = 800usize;
    let mut samples = Vec::with_capacity(steps + 1);
    for index in 0..=steps {
        let t = index as f64 / steps as f64;
        let x = lo + (hi - lo) * t;
        let y = eval(expr, Some(x)).unwrap_or(f64::NAN);
        samples.push((x, y));
    }
    if samples.iter().all(|(_, y)| y.is_finite() && y.abs() < 1e-7) {
        return Numeric::All;
    }
    let mut roots = Vec::new();
    for pair in samples.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if !y0.is_finite() || !y1.is_finite() {
            continue;
        }
        if y0.abs() < 1e-10 {
            roots.push(x0);
            continue;
        }
        if y0.signum() != y1.signum() && y0.abs() < 20.0 && y1.abs() < 20.0 {
            if let Some(root) = bisect(expr, x0, x1, y0, y1) {
                roots.push(root);
            }
        }
    }
    for index in 1..samples.len().saturating_sub(1) {
        let (_, left) = samples[index - 1];
        let (x, y) = samples[index];
        let (_, right) = samples[index + 1];
        if !y.is_finite() || !left.is_finite() || !right.is_finite() {
            continue;
        }
        if y.abs() <= left.abs() && y.abs() <= right.abs() && y.abs() < 2e-2 {
            if let Some(root) = refine_root(expr, x) {
                roots.push(root);
            }
        }
    }
    roots.retain(|root| root.is_finite() && (lo - 1e-6..=hi + 1e-6).contains(root));
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut unique = Vec::new();
    for root in roots {
        if unique
            .last()
            .is_none_or(|previous: &f64| (root - previous).abs() > 1e-4)
        {
            unique.push(root);
        }
    }
    if unique.len() > 16 {
        unique.truncate(16);
    }
    Numeric::Roots(unique)
}

fn bisect(expr: &Expr, mut left: f64, mut right: f64, mut y_left: f64, mut y_right: f64) -> Option<f64> {
    for _ in 0..60 {
        let mid = (left + right) / 2.0;
        let y_mid = eval(expr, Some(mid)).ok()?;
        if !y_mid.is_finite() {
            return None;
        }
        if y_mid.abs() < 1e-10 || (right - left).abs() < 1e-10 {
            return Some(mid);
        }
        if y_left.signum() != y_mid.signum() {
            right = mid;
            y_right = y_mid;
        } else {
            left = mid;
            y_left = y_mid;
        }
    }
    let _ = y_right;
    Some((left + right) / 2.0)
}

fn refine_root(expr: &Expr, mut x: f64) -> Option<f64> {
    for _ in 0..12 {
        let y = eval(expr, Some(x)).ok()?;
        if !y.is_finite() {
            return None;
        }
        if y.abs() < 1e-8 {
            return Some(x);
        }
        let step = (1e-6 * (1.0 + x.abs())).max(1e-8);
        let ahead = eval(expr, Some(x + step)).ok()?;
        let slope = (ahead - y) / step;
        if !slope.is_finite() || slope.abs() < 1e-12 {
            return (y.abs() < 1e-6).then_some(x);
        }
        let next = x - y / slope;
        if !next.is_finite() || (next - x).abs() > 2.0 {
            return None;
        }
        x = next;
    }
    let y = eval(expr, Some(x)).ok()?;
    (y.abs() < 1e-7).then_some(x)
}

fn polynomial(expr: &Expr) -> Option<Vec<f64>> {
    if !expr.uses_x() {
        let value = eval(expr, None).ok()?;
        return value.is_finite().then_some(vec![value]);
    }
    match expr {
        Expr::Var => Some(vec![0.0, 1.0]),
        Expr::Neg(inner) => {
            let poly = polynomial(inner)?;
            Some(trim_poly(poly.into_iter().map(|coef| -coef).collect()))
        }
        Expr::Add(left, right) => Some(poly_add(&polynomial(left)?, &polynomial(right)?)),
        Expr::Sub(left, right) => {
            let right = polynomial(right)?.into_iter().map(|coef| -coef).collect::<Vec<_>>();
            Some(poly_add(&polynomial(left)?, &right))
        }
        Expr::Mul(left, right) => poly_mul(&polynomial(left)?, &polynomial(right)?),
        Expr::Div(left, right) => {
            let denom = polynomial(right)?;
            if denom.len() != 1 || denom[0].abs() < 1e-12 {
                return None;
            }
            let numer = polynomial(left)?;
            Some(trim_poly(numer.into_iter().map(|coef| coef / denom[0]).collect()))
        }
        Expr::Pow(base, exp) => {
            if exp.uses_x() {
                return None;
            }
            let power = eval(exp, None).ok()?;
            let rounded = power.round();
            if (power - rounded).abs() > 1e-8 || !(0.0..=8.0).contains(&rounded) {
                return None;
            }
            poly_pow(&polynomial(base)?, rounded as u32)
        }
        Expr::Call(_, _) | Expr::Num(_) | Expr::Index(_) | Expr::Sum { .. } | Expr::Prod { .. } | Expr::Limit { .. } => {
            None
        }
    }
}

fn poly_add(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; left.len().max(right.len())];
    for (index, coef) in left.iter().enumerate() {
        out[index] += coef;
    }
    for (index, coef) in right.iter().enumerate() {
        out[index] += coef;
    }
    trim_poly(out)
}

fn poly_mul(left: &[f64], right: &[f64]) -> Option<Vec<f64>> {
    if left.is_empty() || right.is_empty() || left.len() + right.len() > 10 {
        return None;
    }
    let mut out = vec![0.0; left.len() + right.len() - 1];
    for (i, left_coef) in left.iter().enumerate() {
        for (j, right_coef) in right.iter().enumerate() {
            out[i + j] += left_coef * right_coef;
        }
    }
    Some(trim_poly(out))
}

fn poly_pow(base: &[f64], exp: u32) -> Option<Vec<f64>> {
    let mut acc = vec![1.0];
    for _ in 0..exp {
        acc = poly_mul(&acc, base)?;
    }
    Some(trim_poly(acc))
}

fn trim_poly(mut coeffs: Vec<f64>) -> Vec<f64> {
    while coeffs.len() > 1 && coeffs.last().is_some_and(|coef| coef.abs() < 1e-9) {
        coeffs.pop();
    }
    if coeffs.iter().all(|coef| coef.abs() < 1e-9) {
        vec![0.0]
    } else {
        coeffs
    }
}

fn solve_poly(coeffs: &[f64]) -> PolySolve {
    let coeffs = trim_poly(coeffs.to_vec());
    match coeffs.len() {
        0 | 1 if coeffs.first().copied().unwrap_or(0.0).abs() < 1e-8 => PolySolve::All,
        1 => PolySolve::None,
        2 => {
            let slope = coeffs[1];
            if slope.abs() < 1e-12 {
                return PolySolve::Defer;
            }
            PolySolve::Roots(vec![-coeffs[0] / slope])
        }
        3 => {
            let (c, b, a) = (coeffs[0], coeffs[1], coeffs[2]);
            if a.abs() < 1e-12 {
                return PolySolve::Defer;
            }
            let disc = b * b - 4.0 * a * c;
            if disc < -1e-8 {
                PolySolve::None
            } else if disc.abs() <= 1e-8 {
                PolySolve::Roots(vec![-b / (2.0 * a)])
            } else {
                let root = disc.sqrt();
                let mut values = vec![(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)];
                values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
                PolySolve::Roots(values)
            }
        }
        _ => PolySolve::Defer,
    }
}

/// Break the polyline when a step is non-finite or huge compared with the
/// typical step, and when a large value flips sign (vertical asymptote).
pub fn split_on_discontinuity(points: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let mut deltas = Vec::new();
    for pair in points.windows(2) {
        if pair[0].1.is_finite() && pair[1].1.is_finite() {
            deltas.push((pair[1].1 - pair[0].1).abs());
        }
    }
    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if deltas.is_empty() {
        0.0
    } else {
        deltas[deltas.len() / 2]
    };
    let threshold = (median * 50.0).max(40.0);

    let mut lines = Vec::new();
    let mut current = Vec::new();
    for &(x, y) in points {
        if !x.is_finite() || !y.is_finite() {
            flush_line(&mut current, &mut lines);
            continue;
        }
        if let Some(&(_, previous)) = current.last() {
            let dy = (y - previous).abs();
            let opposite = previous.signum() != y.signum() && previous.abs() > 25.0 && y.abs() > 25.0;
            if dy > threshold || opposite {
                flush_line(&mut current, &mut lines);
            }
        }
        current.push((x, y));
    }
    flush_line(&mut current, &mut lines);
    lines
}

fn flush_line(current: &mut Vec<(f64, f64)>, lines: &mut Vec<Vec<(f64, f64)>>) {
    if current.len() >= 2 {
        lines.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

pub fn sample_expr(expr: &Expr, x_min: f64, x_max: f64, samples: usize) -> Vec<Vec<(f64, f64)>> {
    if !x_min.is_finite() || !x_max.is_finite() || samples < 2 {
        return Vec::new();
    }
    let (x_min, x_max) = if x_min <= x_max {
        (x_min, x_max)
    } else {
        (x_max, x_min)
    };
    if x_max - x_min < 1e-15 {
        return Vec::new();
    }
    let samples = samples.clamp(2, 2000);
    let mut points = Vec::with_capacity(samples);
    for index in 0..samples {
        let t = index as f64 / (samples as f64 - 1.0);
        let x = x_min + (x_max - x_min) * t;
        let y = match eval(expr, Some(x)) {
            Ok(value) if value.is_finite() => value,
            _ => f64::NAN,
        };
        points.push((x, y));
    }
    split_on_discontinuity(&points)
}

pub fn sample_expression_source(
    source: &str,
    x_min: f64,
    x_max: f64,
    samples: usize,
) -> Vec<Vec<[f64; 2]>> {
    let Ok(expr) = parse_expression(source) else {
        return Vec::new();
    };
    sample_expr(&expr, x_min, x_max, samples)
        .into_iter()
        .map(|line| line.into_iter().map(|(x, y)| [x, y]).collect())
        .collect()
}

pub fn parse_with_names(input: &str, names: &[String]) -> Result<Expr, String> {
    let normalized = wolfram_normalize(input);
    let mut parser = Parser {
        s: &normalized,
        i: 0,
        bounds: names.iter().map(|name| name.to_ascii_lowercase()).collect(),
    };
    let expr = parser.parse_expr()?;
    parser.skip_ws();
    if parser.i != parser.s.len() {
        return Err("Unexpected input".into());
    }
    Ok(expr)
}

pub fn eval_with(expr: &Expr, x: Option<f64>, locals: &[(&str, f64)]) -> Result<f64, String> {
    eval_in(expr, x, locals)
}

pub fn format_value(value: f64) -> String {
    if !value.is_finite() {
        return "undefined".into();
    }
    let rounded = (value * 1e10).round() / 1e10;
    let mut text = format!("{rounded}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

fn in_degrees(expr: Expr) -> Expr {
    match expr {
        Expr::Call(func, arg) => {
            let arg = Box::new(in_degrees(*arg));
            let scale = Expr::Div(Box::new(Expr::Num(PI)), Box::new(Expr::Num(180.0)));
            match func {
                Func::Sin | Func::Cos | Func::Tan | Func::Sec | Func::Csc | Func::Cot => Expr::Call(
                    func,
                    Box::new(Expr::Mul(arg, Box::new(scale))),
                ),
                Func::Asin | Func::Acos | Func::Atan | Func::Asec | Func::Acsc | Func::Acot => {
                    Expr::Mul(Box::new(Expr::Call(func, arg)), Box::new(Expr::Div(Box::new(Expr::Num(180.0)), Box::new(Expr::Num(PI)))))
                }
                other => Expr::Call(other, arg),
            }
        }
        Expr::Neg(inner) => Expr::Neg(Box::new(in_degrees(*inner))),
        Expr::Add(a, b) => Expr::Add(Box::new(in_degrees(*a)), Box::new(in_degrees(*b))),
        Expr::Sub(a, b) => Expr::Sub(Box::new(in_degrees(*a)), Box::new(in_degrees(*b))),
        Expr::Mul(a, b) => Expr::Mul(Box::new(in_degrees(*a)), Box::new(in_degrees(*b))),
        Expr::Div(a, b) => Expr::Div(Box::new(in_degrees(*a)), Box::new(in_degrees(*b))),
        Expr::Pow(a, b) => Expr::Pow(Box::new(in_degrees(*a)), Box::new(in_degrees(*b))),
        Expr::Sum { var, lower, upper, body } => Expr::Sum {
            var,
            lower: Box::new(in_degrees(*lower)),
            upper: Box::new(in_degrees(*upper)),
            body: Box::new(in_degrees(*body)),
        },
        Expr::Prod { var, lower, upper, body } => Expr::Prod {
            var,
            lower: Box::new(in_degrees(*lower)),
            upper: Box::new(in_degrees(*upper)),
            body: Box::new(in_degrees(*body)),
        },
        Expr::Limit { var, target, body } => Expr::Limit {
            var,
            target: Box::new(in_degrees(*target)),
            body: Box::new(in_degrees(*body)),
        },
        other => other,
    }
}

fn prepare_expr(source: &str, names: &[String], degrees: bool) -> Result<Expr, String> {
    let expr = parse_with_names(source, names)?;
    Ok(if degrees { in_degrees(expr) } else { expr })
}

fn split_top(input: &str, op: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            found if found == op && depth == 0 => {
                parts.push(input[start..index].trim().to_string());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(input[start..].trim().to_string());
    parts
}

fn split_relation(input: &str) -> Option<(String, String, String)> {
    let mut depth = 0i32;
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut index = 0usize;
    while index < chars.len() {
        let (byte, ch) = chars[index];
        match ch {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            '<' | '>' | '=' if depth == 0 => {
                let next = chars.get(index + 1).map(|(_, ch)| *ch);
                let (op, width) = if ch != '=' && next == Some('=') {
                    (format!("{ch}="), 2)
                } else if ch == '=' {
                    ("=".into(), 1)
                } else {
                    (ch.to_string(), 1)
                };
                let left = input[..byte].trim().to_string();
                let right_at = chars.get(index + width).map(|(byte, _)| *byte).unwrap_or(input.len());
                let right = input[right_at..].trim().to_string();
                if left.is_empty() || right.is_empty() {
                    return None;
                }
                return Some((left, op, right));
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn eval_source(source: &str, names: &[String], locals: &[(&str, f64)], x: Option<f64>, degrees: bool) -> Result<f64, String> {
    let expr = prepare_expr(source, names, degrees)?;
    eval_with(&expr, x, locals)
}

fn relation_holds(source: &str, names: &[String], locals: &[(&str, f64)], x: Option<f64>, degrees: bool) -> bool {
    let Some((left, op, right)) = split_relation(source) else {
        return eval_source(source, names, locals, x, degrees)
            .ok()
            .is_some_and(|value| value.is_finite() && value != 0.0);
    };
    let Ok(left) = eval_source(&left, names, locals, x, degrees) else {
        return false;
    };
    let Ok(right) = eval_source(&right, names, locals, x, degrees) else {
        return false;
    };
    if !left.is_finite() || !right.is_finite() {
        return false;
    }
    match op.as_str() {
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        "=" => (left - right).abs() <= 1e-8 * (1.0 + left.abs().max(right.abs())),
        _ => false,
    }
}

fn to_lines(lines: Vec<Vec<(f64, f64)>>) -> Vec<Vec<[f64; 2]>> {
    lines
        .into_iter()
        .map(|line| line.into_iter().map(|(x, y)| [x, y]).collect())
        .collect()
}

fn sample_prepared(
    expr: &Expr,
    x_min: f64,
    x_max: f64,
    samples: usize,
    locals: &[(&str, f64)],
    domain: Option<&str>,
    names: &[String],
    degrees: bool,
) -> Vec<Vec<[f64; 2]>> {
    if !x_min.is_finite() || !x_max.is_finite() || samples < 2 {
        return Vec::new();
    }
    let samples = samples.clamp(2, 2000);
    let mut points = Vec::with_capacity(samples);
    for index in 0..samples {
        let t = index as f64 / (samples as f64 - 1.0);
        let x = x_min + (x_max - x_min) * t;
        let y = match eval_with(expr, Some(x), locals) {
            Ok(value) if value.is_finite() => value,
            _ => f64::NAN,
        };
        let y = if y.is_finite() {
            if let Some(domain) = domain.filter(|item| !item.trim().is_empty()) {
                if relation_holds(domain, names, locals, Some(x), degrees) {
                    y
                } else {
                    f64::NAN
                }
            } else {
                y
            }
        } else {
            y
        };
        points.push((x, y));
    }
    to_lines(split_on_discontinuity(&points))
}

fn sample_piecewise(
    source: &str,
    names: &[String],
    locals: &[(&str, f64)],
    x_min: f64,
    x_max: f64,
    samples: usize,
    domain: Option<&str>,
    degrees: bool,
) -> Vec<Vec<[f64; 2]>> {
    let body = source.trim().trim_start_matches('{').trim_end_matches('}').trim();
    let arms = split_top(body, ',');
    let parsed = arms
        .iter()
        .filter_map(|arm| {
            let (cond, expr) = if let Some((cond, expr)) = arm.split_once(':') {
                (Some(cond.trim()), expr.trim())
            } else {
                (None, arm.trim())
            };
            if expr.is_empty() {
                return None;
            }
            prepare_expr(expr, names, degrees).ok().map(|expr| (cond.map(str::to_string), expr))
        })
        .collect::<Vec<_>>();
    if parsed.is_empty() || samples < 2 {
        return Vec::new();
    }
    let samples = samples.clamp(2, 2000);
    let mut points = Vec::with_capacity(samples);
    for index in 0..samples {
        let t = index as f64 / (samples as f64 - 1.0);
        let x = x_min + (x_max - x_min) * t;
        let mut y = f64::NAN;
        for (cond, expr) in &parsed {
            let ok = match cond {
                Some(cond) if !cond.is_empty() => relation_holds(cond, names, locals, Some(x), degrees),
                _ => true,
            };
            if !ok {
                continue;
            }
            y = eval_with(expr, Some(x), locals).unwrap_or(f64::NAN);
            break;
        }
        if y.is_finite() {
            if let Some(domain) = domain.filter(|item| !item.trim().is_empty()) {
                if !relation_holds(domain, names, locals, Some(x), degrees) {
                    y = f64::NAN;
                }
            }
        }
        points.push((x, y));
    }
    to_lines(split_on_discontinuity(&points))
}

fn sample_parameter(
    x_src: &str,
    y_src: &str,
    names: &[String],
    locals: &[(&str, f64)],
    samples: usize,
    degrees: bool,
    polar: bool,
) -> Vec<Vec<[f64; 2]>> {
    let mut names = names.to_vec();
    for extra in ["t", "theta"] {
        if !names.iter().any(|name| name == extra) {
            names.push((*extra).into());
        }
    }
    let Ok(x_expr) = prepare_expr(x_src, &names, degrees) else {
        return Vec::new();
    };
    let Ok(y_expr) = prepare_expr(if polar { x_src } else { y_src }, &names, degrees) else {
        return Vec::new();
    };
    let samples = samples.clamp(32, 2000);
    let turns = 2.0;
    let mut points = Vec::with_capacity(samples);
    for index in 0..samples {
        let t = turns * PI * 2.0 * (index as f64 / (samples as f64 - 1.0));
        let mut scoped = locals.to_vec();
        scoped.push(("t", t));
        scoped.push(("theta", t));
        if polar {
            let Ok(radius) = eval_with(&x_expr, Some(t), &scoped) else {
                points.push((f64::NAN, f64::NAN));
                continue;
            };
            if !radius.is_finite() {
                points.push((f64::NAN, f64::NAN));
                continue;
            }
            points.push((radius * t.cos(), radius * t.sin()));
        } else {
            let x = eval_with(&x_expr, None, &scoped).unwrap_or(f64::NAN);
            let y = eval_with(&y_expr, None, &scoped).unwrap_or(f64::NAN);
            if x.is_finite() && y.is_finite() {
                points.push((x, y));
            } else {
                points.push((f64::NAN, f64::NAN));
            }
        }
    }
    to_lines(split_on_discontinuity(&points))
}

fn sample_implicit_grid(
    source: &str,
    names: &[String],
    locals: &[(&str, f64)],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    degrees: bool,
    region: bool,
) -> Vec<Vec<[f64; 2]>> {
    let mut names = names.to_vec();
    if !names.iter().any(|name| name == "y") {
        names.push("y".into());
    }
    let expr = if region {
        None
    } else {
        Some(match prepare_expr(source, &names, degrees) {
            Ok(expr) => expr,
            Err(_) => return Vec::new(),
        })
    };
    let n = 72usize;
    let mut grid = vec![f64::NAN; (n + 1) * (n + 1)];
    for row in 0..=n {
        let y = y_min + (y_max - y_min) * (row as f64 / n as f64);
        for col in 0..=n {
            let x = x_min + (x_max - x_min) * (col as f64 / n as f64);
            let mut scoped = locals.to_vec();
            scoped.push(("y", y));
            grid[row * (n + 1) + col] = if region {
                if relation_holds(source, &names, &scoped, Some(x), degrees) {
                    1.0
                } else {
                    0.0
                }
            } else {
                eval_with(expr.as_ref().expect("curve"), Some(x), &scoped).unwrap_or(f64::NAN)
            };
        }
    }
    let x_at = |col: usize| x_min + (x_max - x_min) * (col as f64 / n as f64);
    let y_at = |row: usize| y_min + (y_max - y_min) * (row as f64 / n as f64);
    if region {
        let mut cells = Vec::new();
        for row in 0..n {
            for col in 0..n {
                let value = grid[row * (n + 1) + col];
                if value.is_finite() && value > 0.0 {
                    let x0 = x_at(col);
                    let x1 = x_at(col + 1);
                    let y0 = y_at(row);
                    let y1 = y_at(row + 1);
                    cells.push(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
                }
            }
        }
        return cells;
    }
    let mut segments = Vec::new();
    let interp = |a: f64, b: f64, va: f64, vb: f64| {
        let denom = va - vb;
        let t = if denom.abs() < 1e-12 { 0.5 } else { va / denom };
        a + (b - a) * t.clamp(0.0, 1.0)
    };
    for row in 0..n {
        for col in 0..n {
            let i = row * (n + 1) + col;
            let values = [grid[i], grid[i + 1], grid[i + 1 + (n + 1)], grid[i + (n + 1)]];
            if values.iter().any(|value| !value.is_finite()) {
                continue;
            }
            let mut mask = 0u8;
            for (bit, value) in values.iter().enumerate() {
                if *value > 0.0 {
                    mask |= 1 << bit;
                }
            }
            if mask == 0 || mask == 15 {
                continue;
            }
            let x0 = x_at(col);
            let x1 = x_at(col + 1);
            let y0 = y_at(row);
            let y1 = y_at(row + 1);
            let bottom = [interp(x0, x1, values[0], values[1]), y0];
            let right = [x1, interp(y0, y1, values[1], values[2])];
            let top = [interp(x0, x1, values[3], values[2]), y1];
            let left = [x0, interp(y0, y1, values[0], values[3])];
            let edges: &[([f64; 2], [f64; 2])] = match mask {
                1 | 14 => &[(left, bottom)],
                2 | 13 => &[(bottom, right)],
                4 | 11 => &[(right, top)],
                8 | 7 => &[(top, left)],
                3 | 12 => &[(left, right)],
                6 | 9 => &[(bottom, top)],
                5 => &[(left, bottom), (right, top)],
                10 => &[(bottom, right), (top, left)],
                _ => &[],
            };
            for (start, end) in edges {
                segments.push(vec![*start, *end]);
            }
        }
    }
    segments
}

pub fn sample_job(
    plot: &str,
    expression: &str,
    domain: Option<&str>,
    bindings: &[(&str, f64)],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    samples: usize,
    degrees: bool,
) -> Vec<Vec<[f64; 2]>> {
    let names = bindings
        .iter()
        .map(|(name, _)| (*name).to_ascii_lowercase())
        .collect::<Vec<_>>();
    let locals = names
        .iter()
        .zip(bindings.iter())
        .map(|(name, (_, value))| (name.as_str(), *value))
        .collect::<Vec<_>>();
    let plot = if plot.is_empty() { "function" } else { plot };
    match plot {
        "point" => {
            let Some((x_src, y_src)) = expression.split_once("||") else {
                return Vec::new();
            };
            let Ok(x) = eval_source(x_src, &names, &locals, None, degrees) else {
                return Vec::new();
            };
            let Ok(y) = eval_source(y_src, &names, &locals, None, degrees) else {
                return Vec::new();
            };
            if x.is_finite() && y.is_finite() {
                vec![vec![[x, y]]]
            } else {
                Vec::new()
            }
        }
        "polar" => sample_parameter(expression, expression, &names, &locals, samples, degrees, true),
        "parametric" => {
            let Some((x_src, y_src)) = expression.split_once("||") else {
                return Vec::new();
            };
            sample_parameter(x_src, y_src, &names, &locals, samples, degrees, false)
        }
        "implicit" => sample_implicit_grid(expression, &names, &locals, x_min, x_max, y_min, y_max, degrees, false),
        "region" => sample_implicit_grid(expression, &names, &locals, x_min, x_max, y_min, y_max, degrees, true),
        "piecewise" => sample_piecewise(expression, &names, &locals, x_min, x_max, samples, domain, degrees),
        "below" | "above" | "function" => {
            let Ok(expr) = prepare_expr(expression, &names, degrees) else {
                return Vec::new();
            };
            sample_prepared(&expr, x_min, x_max, samples, &locals, domain, &names, degrees)
        }
        _ => Vec::new(),
    }
}

#[derive(Clone, Copy)]
struct Cplx {
    re: f64,
    im: f64,
}

impl Cplx {
    fn real(value: f64) -> Self {
        Self { re: value, im: 0.0 }
    }

    fn add(self, other: Self) -> Self {
        Self { re: self.re + other.re, im: self.im + other.im }
    }

    fn sub(self, other: Self) -> Self {
        Self { re: self.re - other.re, im: self.im - other.im }
    }

    fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    fn div(self, other: Self) -> Self {
        let denom = other.re * other.re + other.im * other.im;
        Self {
            re: (self.re * other.re + self.im * other.im) / denom,
            im: (self.im * other.re - self.re * other.im) / denom,
        }
    }

    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    fn realish(self) -> bool {
        self.im.abs() <= 1e-9 * (1.0 + self.re.abs())
    }
}

fn c_exp(value: Cplx) -> Cplx {
    let scale = value.re.exp();
    Cplx { re: scale * value.im.cos(), im: scale * value.im.sin() }
}

fn c_ln(value: Cplx) -> Cplx {
    Cplx { re: value.abs().ln(), im: value.im.atan2(value.re) }
}

fn c_sqrt(value: Cplx) -> Cplx {
    if value.im.abs() <= 1e-12 && value.re < 0.0 {
        return Cplx { re: 0.0, im: (-value.re).sqrt() };
    }
    let radius = value.abs().sqrt();
    let angle = value.im.atan2(value.re) / 2.0;
    Cplx { re: radius * angle.cos(), im: radius * angle.sin() }
}

fn c_sin(value: Cplx) -> Cplx {
    Cplx {
        re: value.re.sin() * value.im.cosh(),
        im: value.re.cos() * value.im.sinh(),
    }
}

fn c_cos(value: Cplx) -> Cplx {
    Cplx {
        re: value.re.cos() * value.im.cosh(),
        im: -value.re.sin() * value.im.sinh(),
    }
}

fn c_pow(base: Cplx, exp: Cplx) -> Cplx {
    if base.abs() < 1e-15 {
        return if exp.abs() < 1e-15 { Cplx::real(1.0) } else { Cplx::real(0.0) };
    }
    c_exp(exp.mul(c_ln(base)))
}

fn eval_c(expr: &Expr, x: Option<f64>, locals: &[(&str, f64)]) -> Result<Cplx, String> {
    let value = match expr {
        Expr::Num(number) => Cplx::real(*number),
        Expr::Var => Cplx::real(x.ok_or("This expression needs x")?),
        Expr::Index(name) => {
            if let Some((_, value)) = locals.iter().rev().find(|(key, _)| key == name) {
                Cplx::real(*value)
            } else if name == "i" {
                Cplx { re: 0.0, im: 1.0 }
            } else {
                return Err(format!("Unknown name '{name}'"));
            }
        }
        Expr::Neg(inner) => {
            let inner = eval_c(inner, x, locals)?;
            Cplx { re: -inner.re, im: -inner.im }
        }
        Expr::Add(left, right) => eval_c(left, x, locals)?.add(eval_c(right, x, locals)?),
        Expr::Sub(left, right) => eval_c(left, x, locals)?.sub(eval_c(right, x, locals)?),
        Expr::Mul(left, right) => eval_c(left, x, locals)?.mul(eval_c(right, x, locals)?),
        Expr::Div(left, right) => eval_c(left, x, locals)?.div(eval_c(right, x, locals)?),
        Expr::Pow(base, exp) => c_pow(eval_c(base, x, locals)?, eval_c(exp, x, locals)?),
        Expr::Call(func, arg) => call_c(*func, eval_c(arg, x, locals)?)?,
        Expr::Sum { var, lower, upper, body } => {
            return fold_c(var, lower, upper, body, x, locals, true);
        }
        Expr::Prod { var, lower, upper, body } => {
            return fold_c(var, lower, upper, body, x, locals, false);
        }
        Expr::Limit { .. } => {
            let value = eval_in(expr, x, locals)?;
            if !value.is_finite() {
                return Err("That limit is undefined".into());
            }
            Cplx::real(value)
        }
    };
    Ok(value)
}

fn call_c(func: Func, arg: Cplx) -> Result<Cplx, String> {
    let value = match func {
        Func::Sqrt => c_sqrt(arg),
        Func::Abs => Cplx::real(arg.abs()),
        Func::Sin => c_sin(arg),
        Func::Cos => c_cos(arg),
        Func::Tan => c_sin(arg).div(c_cos(arg)),
        Func::Ln => c_ln(arg),
        Func::Log => c_ln(arg).div(c_ln(Cplx::real(10.0))),
        Func::Log2 => c_ln(arg).div(c_ln(Cplx::real(2.0))),
        _ if arg.realish() => {
            let value = func.apply(arg.re);
            if !value.is_finite() {
                return Err("That value is undefined".into());
            }
            Cplx::real(value)
        }
        _ => return Err("That function is not available for complex numbers yet".into()),
    };
    if !value.re.is_finite() || !value.im.is_finite() {
        return Err("That value is undefined".into());
    }
    Ok(value)
}

fn fold_c(
    var: &str,
    lower: &Expr,
    upper: &Expr,
    body: &Expr,
    x: Option<f64>,
    locals: &[(&str, f64)],
    sum: bool,
) -> Result<Cplx, String> {
    let low = eval_c(lower, x, locals)?;
    let high = eval_c(upper, x, locals)?;
    if !low.realish() || !high.realish() {
        return Err("Sum and product limits should be integers".into());
    }
    let low = as_bound(low.re).ok_or("Sum and product limits should be integers")?;
    let high = as_bound(high.re).ok_or("Sum and product limits should be integers")?;
    if high < low {
        return Ok(if sum { Cplx::real(0.0) } else { Cplx::real(1.0) });
    }
    let count = (high as i128 - low as i128 + 1).unsigned_abs();
    if count > 20_000 {
        return Err("That sum is too long".into());
    }
    let mut acc = if sum { Cplx::real(0.0) } else { Cplx::real(1.0) };
    for step in low..=high {
        let mut next = locals.to_vec();
        next.push((var, step as f64));
        let value = eval_c(body, x, &next)?;
        acc = if sum { acc.add(value) } else { acc.mul(value) };
    }
    Ok(acc)
}

pub fn present_value(
    expression: &str,
    bindings: &[(&str, f64)],
    degrees: bool,
    mode: &str,
) -> Result<String, String> {
    let names = bindings
        .iter()
        .map(|(name, _)| (*name).to_ascii_lowercase())
        .collect::<Vec<_>>();
    let locals = names
        .iter()
        .zip(bindings.iter())
        .map(|(name, (_, value))| (name.as_str(), *value))
        .collect::<Vec<_>>();
    let expr = prepare_expr(expression, &names, degrees)?;
    if expr.uses_x() {
        return Err("This expression still depends on x".into());
    }
    if mode == "complex" {
        let value = eval_c(&expr, None, &locals)?;
        if !value.re.is_finite() || !value.im.is_finite() {
            return Err("That value is undefined".into());
        }
        return Ok(format_complex(value, mode));
    }
    let value = eval_with(&expr, None, &locals)?;
    if !value.is_finite() {
        return Err("That value is undefined".into());
    }
    Ok(format_real(value, mode))
}

pub fn format_real(value: f64, mode: &str) -> String {
    if mode == "integer" {
        if !value.is_finite() {
            return "undefined".into();
        }
        return format!("{}", value.round() as i64);
    }
    if mode == "irrational" || mode == "complex" {
        if let Some(exact) = format_exact(value) {
            return exact;
        }
    }
    format_value(value)
}

fn format_complex(value: Cplx, mode: &str) -> String {
    let mut re = value.re;
    let mut im = value.im;
    if mode == "integer" {
        re = re.round();
        im = im.round();
    }
    if im.abs() <= 1e-9 * (1.0 + re.abs()) {
        return format_real(re, if mode == "integer" { "integer" } else { "irrational" });
    }
    let part_mode = if mode == "integer" { "integer" } else { "irrational" };
    if re.abs() <= 1e-9 {
        return imag_term(im, part_mode);
    }
    let left = format_real(re, part_mode);
    let right = imag_term(im.abs(), part_mode);
    if im < 0.0 {
        format!("{left}-{right}")
    } else {
        format!("{left}+{right}")
    }
}

fn imag_term(value: f64, mode: &str) -> String {
    let shown = format_real(value.abs(), mode);
    let body = if shown == "1" {
        "i".to_string()
    } else if shown.chars().all(|ch| ch.is_ascii_digit()) {
        format!("{shown}i")
    } else {
        format!("({shown})i")
    };
    if value < 0.0 {
        format!("-{body}")
    } else {
        body
    }
}

fn format_exact(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    if value.abs() < 1e-10 {
        return Some("0".into());
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let magnitude = value.abs();
    if let Some(text) = exact_integer(magnitude) {
        return Some(format!("{sign}{text}"));
    }
    if let Some(text) = unit_multiple(magnitude / PI, "pi") {
        return Some(format!("{sign}{text}"));
    }
    if (magnitude - E).abs() <= 1e-8 * (1.0 + magnitude) {
        return Some(format!("{sign}e"));
    }
    for n in 2..=48 {
        if !square_free(n) {
            continue;
        }
        if let Some(text) = unit_multiple(magnitude / (n as f64).sqrt(), &format!("sqrt({n})")) {
            return Some(format!("{sign}{text}"));
        }
    }
    let (numer, denom) = rational_pair(magnitude, 12)?;
    if denom == 1 {
        return Some(format!("{sign}{numer}"));
    }
    Some(format!("{sign}{numer}/{denom}"))
}

fn exact_integer(value: f64) -> Option<String> {
    let rounded = value.round();
    if rounded.abs() >= 1e15 {
        return None;
    }
    ((value - rounded).abs() <= 1e-8 * (1.0 + value.abs())).then(|| format!("{}", rounded as i64))
}

fn unit_multiple(value: f64, unit: &str) -> Option<String> {
    let (numer, denom) = rational_pair(value, 12)?;
    if numer == 0 {
        return None;
    }
    let numer = numer.abs();
    Some(match (numer, denom) {
        (1, 1) => unit.to_string(),
        (_, 1) => format!("{numer}*{unit}"),
        (1, _) => format!("{unit}/{denom}"),
        _ => format!("({numer}/{denom})*{unit}"),
    })
}

fn square_free(n: i32) -> bool {
    let mut value = n;
    let mut prime = 2;
    while prime * prime <= value {
        if value % (prime * prime) == 0 {
            return false;
        }
        if value % prime == 0 {
            value /= prime;
        }
        prime += 1;
    }
    true
}

pub fn eval_graph_value(expression: &str, bindings: &[(&str, f64)], degrees: bool) -> Result<f64, String> {
    let names = bindings
        .iter()
        .map(|(name, _)| (*name).to_ascii_lowercase())
        .collect::<Vec<_>>();
    let locals = names
        .iter()
        .zip(bindings.iter())
        .map(|(name, (_, value))| (name.as_str(), *value))
        .collect::<Vec<_>>();
    let expr = prepare_expr(expression, &names, degrees)?;
    if expr.uses_x() {
        return Err("This expression still depends on x".into());
    }
    let value = eval_with(&expr, None, &locals)?;
    if !value.is_finite() {
        return Err("That value is undefined".into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual} != {expected}"
        );
    }

    fn value(source: &str) -> f64 {
        let expr = parse_expression(source).unwrap();
        eval(&expr, None).unwrap()
    }

    fn at(source: &str, x: f64) -> f64 {
        let expr = parse_expression(source).unwrap();
        eval(&expr, Some(x)).unwrap()
    }

    #[test]
    fn arithmetic_and_precedence() {
        approx(value("2+2"), 4.0);
        approx(value("2+3*4"), 14.0);
        approx(value("(2+3)*4"), 20.0);
        approx(value("8/4/2"), 1.0);
        approx(value("2^10"), 1024.0);
        approx(value("2**3**2"), 512.0);
        approx(value("2^3^2"), 512.0);
        approx(value("-3^2"), -9.0);
        approx(value("(-3)^2"), 9.0);
        approx(value("2+-3"), -1.0);
        approx(value("--3"), 3.0);
        approx(value("1e-2"), 0.01);
        approx(value("1e2"), 100.0);
        approx(value("2e"), 2.0 * E);
    }

    #[test]
    fn functions_constants_and_implicit_multiplication() {
        approx(value("sqrt(9)+1"), 4.0);
        approx(value("sqrt(2)").powi(2), 2.0);
        approx(value("sin(pi/2)"), 1.0);
        approx(value("sin pi"), 0.0);
        approx(value("cos(0)"), 1.0);
        approx(value("ln(e)"), 1.0);
        approx(value("ln(e^2)"), 2.0);
        approx(value("log(e)"), 1.0);
        approx(value("log10(100)"), 2.0);
        approx(value("log10(1e3)"), 3.0);
        approx(value("exp(1)"), E);
        approx(value("abs(-3.5)"), 3.5);
        approx(value("abs(1-4)"), 3.0);
        approx(value("2(3+4)"), 14.0);
        approx(value("(2+3)(4)"), 20.0);
        approx(value("2pi"), 2.0 * PI);
        approx(at("2x+1", 3.0), 7.0);
        approx(at("sin(x)+cos(x)", 0.0), 1.0);
        approx(value("π"), PI);
        approx(value("2×3÷2"), 3.0);
    }

    #[test]
    fn rejects_unknown_names_and_trailing_junk() {
        assert!(parse_expression("chrome").is_err());
        assert!(parse_expression("2+2)").is_err());
        assert!(parse_expression("sin").is_err());
        assert!(parse_expression("").is_err());
    }

    #[test]
    fn y_equals_and_multiple_pieces() {
        let pieces = parse_math_input("y = x^2; sin(x); sqrt(9)").unwrap();
        assert_eq!(pieces.len(), 3);
        assert!(pieces[0].uses_x);
        assert_eq!(pieces[0].expression, "x^2");
        assert!(pieces[1].uses_x);
        assert!(!pieces[2].uses_x);
        approx(pieces[2].value.unwrap(), 3.0);

        let prefixed = parse_math_input("=2+2").unwrap();
        approx(prefixed[0].value.unwrap(), 4.0);
        assert!(parse_math_input("a=b").is_err());
    }

    #[test]
    fn equations_report_solutions_and_latex() {
        let vertical = parse_math_input("x=3").unwrap();
        assert_eq!(vertical[0].plots[0].plot, "vertical");
        assert_eq!(vertical[0].plots[0].expression, "3");
        assert_eq!(vertical[0].solutions[0].plain, "x = 3");
        assert_eq!(vertical[0].latex, "x = 3");
        assert_eq!(vertical[0].result_latex, "x = 3");
        assert!(vertical[0].uses_x);

        let spaced = parse_math_input("x = 2 + 1").unwrap();
        assert_eq!(spaced[0].solutions[0].plain, "x = 3");
        assert_eq!(spaced[0].latex, "x = 2 + 1");

        let quadratic = parse_math_input("x^2 = 4").unwrap();
        let plains: Vec<_> = quadratic[0].solutions.iter().map(|item| item.plain.as_str()).collect();
        assert_eq!(plains, ["x = -2", "x = 2"]);
        assert_eq!(quadratic[0].plots.len(), 2);
        assert!(quadratic[0].result_latex.contains("x &= -2"));
        assert!(quadratic[0].markers.iter().any(|marker| (marker.x - 2.0).abs() < 1e-8));

        let linear = parse_math_input("2x+1=0").unwrap();
        assert_eq!(linear[0].solutions.len(), 1);
        assert_eq!(linear[0].result_latex, "x = -\\frac{1}{2}");

        let value = parse_math_input("sqrt(9)+1").unwrap();
        assert_eq!(value[0].latex, "\\sqrt{9} + 1");
        assert_eq!(value[0].result_latex, "4");

        let trig = parse_math_input("sin(pi/2)").unwrap();
        assert_eq!(trig[0].latex, "\\sin\\left(\\frac{\\pi}{2}\\right)");
        assert_eq!(trig[0].result_latex, "1");

        let none = parse_math_input("x^2 = -1").unwrap();
        assert!(none[0].solutions.is_empty());
        assert_eq!(none[0].result_plain, "No real solution");

        let identity = parse_math_input("x=x").unwrap();
        assert_eq!(identity[0].result_plain, "True for all real x");
    }

    #[test]
    fn derivatives_and_integrals() {
        let deriv = parse_math_input("d/dx cosx").unwrap();
        assert_eq!(deriv[0].result_label, "Derivative");
        assert!(deriv[0].result_latex.contains("\\sin"));
        let expr = parse_expression(&deriv[0].expression).unwrap();
        approx(eval(&expr, Some(0.0)).unwrap(), 0.0);
        approx(eval(&expr, Some(PI / 2.0)).unwrap(), -1.0);

        let spaced = parse_math_input("d/dx cos(x)").unwrap();
        let expr = parse_expression(&spaced[0].expression).unwrap();
        approx(eval(&expr, Some(PI / 2.0)).unwrap(), -1.0);

        let power = parse_math_input("d/dx x^2").unwrap();
        let expr = parse_expression(&power[0].expression).unwrap();
        approx(eval(&expr, Some(3.0)).unwrap(), 6.0);

        let integral = parse_math_input("=int(sin(x))dx =y").unwrap();
        assert!(integral[0].latex.contains("\\int"));
        assert!(integral[0].latex.contains("= y"));
        assert!(integral[0].result_latex.contains('C'));
        assert_eq!(integral[0].result_label, "Integral");
        assert!(integral[0].uses_x);
        let expr = parse_expression(&integral[0].expression).unwrap();
        approx(eval(&expr, Some(0.0)).unwrap(), -1.0);
        approx(eval(&expr, Some(PI / 2.0)).unwrap(), 0.0);

        let bare = parse_math_input("int(sin(x)) dx").unwrap();
        let expr = parse_expression(&bare[0].expression).unwrap();
        approx(eval(&expr, Some(0.0)).unwrap(), -1.0);
    }

    #[test]
    fn wolfram_input() {
        approx(at("sinx", PI / 2.0), 1.0);
        approx(at("sin2x", PI / 4.0), 1.0);
        approx(at("sinxcosx", 0.0), 0.0);
        approx(at("xsinx", PI / 2.0), PI / 2.0);
        approx(value("sin^2(pi/2)"), 1.0);
        approx(value("log(e)"), 1.0);
        approx(value("exp(1)"), E);

        let spoken = parse_math_input("derivative of cos x").unwrap();
        let expr = parse_expression(&spoken[0].expression).unwrap();
        approx(eval(&expr, Some(PI / 2.0)).unwrap(), -1.0);

        let second = parse_math_input("d^2/dx^2 sinx").unwrap();
        let expr = parse_expression(&second[0].expression).unwrap();
        approx(eval(&expr, Some(PI / 2.0)).unwrap(), -1.0);

        let assigned = parse_math_input("y = int sinx dx").unwrap();
        assert!(assigned[0].latex.starts_with("y ="));
        let expr = parse_expression(&assigned[0].expression).unwrap();
        approx(eval(&expr, Some(0.0)).unwrap(), -1.0);

        let definite = parse_math_input("int sinx dx from 0 to pi").unwrap();
        approx(definite[0].value.unwrap(), 2.0);
        assert_eq!(definite[0].note.as_deref(), Some("Definite integral."));

        let area = parse_math_input("integrate x dx from 0 to 2").unwrap();
        approx(area[0].value.unwrap(), 2.0);

        let solved = parse_math_input("solve x^2=4").unwrap();
        assert_eq!(solved[0].solutions.len(), 2);

        assert!(!looks_like_math("hello world"));
        assert!(!looks_like_math("chrome"));
        assert!(looks_like_math("derivative of cos x"));
    }

    #[test]
    fn inverse_and_special_functions() {
        approx(value("arcsin(0)"), 0.0);
        approx(value("arcsin(1)"), PI / 2.0);
        approx(value("arccos(0)"), PI / 2.0);
        approx(value("arctan(1)"), PI / 4.0);
        approx(value("sinh(0)"), 0.0);
        approx(value("cosh(0)"), 1.0);
        approx(value("floor(1.9)"), 1.0);
        approx(value("ceil(1.1)"), 2.0);
        approx(value("5!"), 120.0);
        approx(value("log2(8)"), 3.0);
        approx(value("sec(0)"), 1.0);
        approx(at("arcsinx", 0.0), 0.0);
        approx(value("sum_{n=1}^{10} n"), 55.0);
        approx(value("prod_{n=1}^{5} n"), 120.0);
        approx(value("lim_{x->0} sin(x)/x"), 1.0);
        let slope = parse_math_input("d/dx arcsin(x)").unwrap();
        let expr = parse_expression(&slope[0].expression).unwrap();
        approx(eval(&expr, Some(0.0)).unwrap(), 1.0);
        assert!(slope[0].latex.contains("\\arcsin"));
        assert!(parse_math_input("sum_{n=1}^{10} n").unwrap()[0].latex.contains("\\sum"));
    }

    #[test]
    fn format_cleans_up_float_noise() {
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number((PI).sin()), "0");
        assert_eq!(format_number(1.0 / 0.0), "∞");
        assert_eq!(format_number(f64::NAN), "undefined");
        assert_eq!(format_number(1024.0), "1024");
    }

    #[test]
    fn smooth_curves_stay_one_polyline() {
        let parabola = parse_expression("x^2").unwrap();
        let lines = sample_expr(&parabola, -5.0, 5.0, 200);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].len() > 100);

        let steep = parse_expression("100*x").unwrap();
        assert_eq!(sample_expr(&steep, -2.0, 2.0, 80).len(), 1);

        let wide = parse_expression("x^2").unwrap();
        assert_eq!(sample_expr(&wide, -100.0, 100.0, 32).len(), 1);

        let sine = parse_expression("sin(x)").unwrap();
        assert_eq!(sample_expr(&sine, -10.0, 10.0, 200).len(), 1);
    }

    #[test]
    fn asymptotes_do_not_draw_across_the_gap() {
        let reciprocal = parse_expression("1/x").unwrap();
        let lines = sample_expr(&reciprocal, -2.0, 2.0, 401);
        assert!(lines.len() >= 2, "expected a break at x = 0, got {}", lines.len());
        assert_no_sign_bridge(&lines);

        let tangent = parse_expression("tan(x)").unwrap();
        let lines = sample_expr(&tangent, 0.0, 3.0, 700);
        assert!(lines.len() >= 2, "expected a break near pi/2, got {}", lines.len());
        assert_no_sign_bridge(&lines);
    }

    #[test]
    fn splitter_breaks_nan_and_huge_jumps() {
        let points = [
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, f64::NAN),
            (3.0, 1.0),
            (4.0, 0.0),
        ];
        assert_eq!(split_on_discontinuity(&points).len(), 2);

        let jump = [
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 2.0),
            (3.0, 1000.0),
            (4.0, -1000.0),
            (5.0, 0.0),
        ];
        let lines = split_on_discontinuity(&jump);
        assert!(lines.len() >= 2);
        assert_no_sign_bridge(&lines);

        let smooth: Vec<_> = (0..20).map(|i| (i as f64, (i as f64).sin())).collect();
        assert_eq!(split_on_discontinuity(&smooth).len(), 1);
    }

    #[test]
    fn slider_polar_and_implicit_curves() {
        let value = eval_graph_value("a*x", &[("a", 3.0)], false);
        assert!(value.is_err());
        approx(eval_graph_value("a+1", &[("a", 3.0)], false).unwrap(), 4.0);
        approx(eval_graph_value("sin(90)", &[], true).unwrap(), 1.0);

        let circle = sample_job(
            "implicit",
            "x^2+y^2-4",
            None,
            &[],
            -4.0,
            4.0,
            -4.0,
            4.0,
            80,
            false,
        );
        assert!(circle.iter().flatten().any(|point| (point[0].hypot(point[1]) - 2.0).abs() < 0.35));

        let polar = sample_job("polar", "2", None, &[], -4.0, 4.0, -4.0, 4.0, 80, false);
        let point = polar.iter().flatten().next().unwrap();
        approx(point[0].hypot(point[1]), 2.0);

        let points = sample_job("point", "1||3", None, &[], -4.0, 4.0, -4.0, 4.0, 20, false);
        assert_eq!(points, vec![vec![[1.0, 3.0]]]);

        let piece = sample_job("piecewise", "x<0:-x,x", None, &[], -2.0, 2.0, -4.0, 4.0, 5, false);
        let flat = piece.into_iter().flatten().collect::<Vec<_>>();
        assert!(flat.iter().all(|point| point[1] >= -1e-9));

        let curve = sample_job("function", "ax^2", None, &[("a", 2.0)], -2.0, 2.0, -8.0, 8.0, 5, false);
        let at_two = curve.iter().flatten().find(|point| (point[0] - 2.0).abs() < 1e-9);
        assert!(at_two.is_some_and(|point| (point[1] - 8.0).abs() < 1e-6));
    }

    #[test]
    fn bounds_and_number_modes() {
        let integral = parse_math_input("int_{0}^{1} (x) dx").unwrap();
        approx(integral[0].value.unwrap(), 0.5);
        let summed = parse_expression("sum_{n=1}^{4} n").unwrap();
        approx(eval(&summed, None).unwrap(), 10.0);
        let plain_bounds = parse_expression("sum_{1}^{10} (n)").unwrap();
        approx(eval(&plain_bounds, None).unwrap(), 55.0);
        assert_eq!(present_value("sqrt(2)", &[], false, "irrational").unwrap(), "sqrt(2)");
        assert_eq!(present_value("pi/2", &[], false, "irrational").unwrap(), "pi/2");
        assert_eq!(present_value("1.6", &[], false, "integer").unwrap(), "2");
        assert_eq!(present_value("sqrt(-1)", &[], false, "complex").unwrap(), "i");
        assert_eq!(present_value("i^2", &[], false, "complex").unwrap(), "-1");
    }

    fn assert_no_sign_bridge(lines: &[Vec<(f64, f64)>]) {
        for line in lines {
            let positive = line.iter().any(|point| point.1 > 20.0);
            let negative = line.iter().any(|point| point.1 < -20.0);
            assert!(
                !(positive && negative),
                "a polyline crossed from a large positive y to a large negative y"
            );
        }
    }
}
