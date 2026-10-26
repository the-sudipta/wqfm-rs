//! Messages for people, on standard error: a header, an aligned summary,
//! warnings, errors with a hint, and a progress bar. Results stay on
//! standard output or in files, unchanged. Colours are dropped when standard
//! error is not a terminal or `NO_COLOR` is set (anstream).

use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use anstyle::{AnsiColor, Style};

static QUIET: AtomicBool = AtomicBool::new(false);

const TITLE: Style = AnsiColor::Cyan.on_default().bold();
const LABEL: Style = AnsiColor::BrightBlack.on_default();
const GOOD: Style = AnsiColor::Green.on_default().bold();
const WARN: Style = AnsiColor::Yellow.on_default().bold();
const ERROR: Style = AnsiColor::Red.on_default().bold();
const HINT: Style = AnsiColor::Cyan.on_default();

/// Turns off the header, summary and progress bar (errors still show).
pub fn set_quiet(quiet: bool) {
    QUIET.store(quiet, Ordering::Relaxed);
}

fn quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

/// `wqfm-rs 0.1.0  <command>`
pub fn header(command: &str) {
    if !quiet() {
        anstream::eprintln!(
            "{TITLE}wqfm-rs {}{TITLE:#}  {LABEL}{command}{LABEL:#}",
            env!("CARGO_PKG_VERSION")
        );
    }
}

/// One aligned line of the summary: `  label   value`.
pub fn line(label: &str, value: &str) {
    if !quiet() {
        anstream::eprintln!("  {LABEL}{label:<9}{LABEL:#}{value}");
    }
}

/// The last line: `  done      in 0.42 s`.
pub fn done(started: Instant) {
    if !quiet() {
        anstream::eprintln!(
            "  {LABEL}{:<9}{LABEL:#}{GOOD}in {}{GOOD:#}",
            "done",
            seconds(started.elapsed().as_secs_f64())
        );
    }
}

pub fn warning(text: &str) {
    anstream::eprintln!("{WARN}warning:{WARN:#} {text}");
}

/// `error: <message>` and, when one fits, `hint: <what to do>`.
pub fn error(message: &str) {
    anstream::eprintln!("{ERROR}error:{ERROR:#} {message}");
    if let Some(hint) = hint_for(message) {
        anstream::eprintln!("{HINT}hint:{HINT:#}  {hint}");
    }
}

fn hint_for(message: &str) -> Option<&'static str> {
    let hints = [
        (
            "an unresolved 4-set",
            "lines like ((a,b,c,d)); n come from generate_wqrts.py on gene trees with \
             polytomies; make the file with `wqfm-rs gt2wq`, which leaves them out",
        ),
        (
            "quartets after an empty line",
            "remove the empty line; wQFM would silently ignore everything after it",
        ),
        (
            "not a finite number",
            "weights must be ordinary numbers such as 3, 0.5 or 2e-3",
        ),
        (
            "is not a number",
            "the last field of each line is the weight, e.g. ((a,b),(c,d)); 0.75",
        ),
        (
            "expected ((a,b),(c,d)); w",
            "each line needs four names and a weight: ((a,b),(c,d)); w",
        ),
        (
            "different taxa",
            "both trees must have exactly the same leaf names",
        ),
        (
            "unknown leaf",
            "the tree has a leaf that does not appear in the quartet file",
        ),
        (
            "a taxon appears twice",
            "each leaf name may appear only once per gene tree",
        ),
        (
            "invalid Newick",
            "each tree must be one line of Newick ending with ;",
        ),
    ];
    hints
        .iter()
        .find(|(pattern, _)| message.contains(pattern))
        .map(|(_, hint)| *hint)
}

/// `1234567` as `1,234,567`.
pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A weight: whole numbers without decimals, others with up to 6.
pub fn weight(w: f64) -> String {
    if w.fract() == 0.0 && w.abs() < 1e15 {
        thousands(w.abs() as usize)
            .chars()
            .rev()
            .chain(if w < 0.0 { Some('-') } else { None })
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    } else {
        format!("{w:.6}")
    }
}

pub fn seconds(s: f64) -> String {
    if s < 10.0 {
        format!("{s:.2} s")
    } else if s < 600.0 {
        format!("{s:.1} s")
    } else {
        format!("{:.0} min {:.0} s", (s / 60.0).floor(), s % 60.0)
    }
}

/// A progress bar on standard error, drawn only on a terminal and only when
/// not quiet; `tick` may be called from several threads.
pub struct Progress {
    total: usize,
    done: AtomicUsize,
    shown: Mutex<usize>,
    active: bool,
    label: &'static str,
}

impl Progress {
    pub fn new(label: &'static str, total: usize) -> Self {
        let active = !quiet() && total > 0 && std::io::stderr().is_terminal();
        Self {
            total,
            done: AtomicUsize::new(0),
            shown: Mutex::new(usize::MAX),
            active,
            label,
        }
    }

    pub fn tick(&self) {
        if !self.active {
            return;
        }
        let d = self.done.fetch_add(1, Ordering::Relaxed) + 1;
        let percent = d * 100 / self.total;
        let mut shown = self.shown.lock().expect("progress lock");
        if *shown == percent {
            return;
        }
        *shown = percent;
        let width = 28;
        let filled = d * width / self.total;
        let bar: String = "#".repeat(filled) + &"-".repeat(width - filled);
        anstream::eprint!(
            "\r  {LABEL}{:<9}{LABEL:#}[{GOOD}{bar}{GOOD:#}] {percent:>3}%  {}/{}",
            self.label,
            thousands(d),
            thousands(self.total)
        );
        let _ = std::io::stderr().flush();
    }

    /// Clears the bar's line.
    pub fn finish(&self) {
        if self.active {
            anstream::eprint!("\r{}\r", " ".repeat(72));
            let _ = std::io::stderr().flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_well() {
        assert_eq!(thousands(1234567), "1,234,567");
        assert_eq!(thousands(12), "12");
        assert_eq!(weight(1650.0), "1,650");
        assert_eq!(weight(-2.0), "-2");
        assert_eq!(weight(0.25), "0.250000");
        assert_eq!(seconds(0.004), "0.00 s");
        assert_eq!(seconds(75.0), "75.0 s");
    }

    #[test]
    fn errors_get_hints() {
        assert!(
            hint_for("line 3: not a quartet ((a,b),(c,d)); w (an unresolved 4-set?)").is_some()
        );
        assert!(hint_for("something else").is_none());
    }
}
