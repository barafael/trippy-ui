//! The columns of the hops table: which exist, which are shown and in what order.
//!
//! Mirrors the terminal frontend's `--tui-custom-columns` model.

use std::fmt::{Display, Formatter};

/// A column of the hops table.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Column {
    /// Time-to-live (1-based).
    Ttl,
    /// Host(s) reported at the hop.
    Host,
    /// Packet loss percentage.
    LossPct,
    /// Probes sent.
    Sent,
    /// Responses received.
    Received,
    /// Round-trip time of the most recent probe.
    Last,
    /// Rolling average round-trip time.
    Average,
    /// Best round-trip time.
    Best,
    /// Worst round-trip time.
    Worst,
    /// Standard deviation of round-trip time.
    StdDev,
    /// Hop status.
    Status,
    /// Current jitter.
    Jitter,
    /// Average jitter.
    Javg,
    /// Worst jitter.
    Jmax,
    /// Smoothed jitter.
    Jinta,
    /// Source port of the last probe.
    LastSrcPort,
    /// Destination port of the last probe.
    LastDestPort,
    /// Sequence number of the last probe.
    LastSeq,
    /// ICMP packet type of the last probe.
    LastIcmpPacketType,
    /// ICMP packet code of the last probe.
    LastIcmpPacketCode,
    /// NAT detection status of the last probe.
    LastNatStatus,
    /// Probes that failed.
    Failed,
    /// Probes with forward loss.
    Floss,
    /// Probes with backward loss.
    Bloss,
    /// Forward loss percentage.
    FlossPct,
    /// Differentiated Services Code Point.
    Dscp,
    /// Explicit Congestion Notification.
    Ecn,
    /// Autonomous system number.
    Asn,
}

/// Every column in the order the settings dialog lists them.
pub const ALL: [Column; 28] = [
    Column::Ttl,
    Column::Host,
    Column::LossPct,
    Column::Sent,
    Column::Received,
    Column::Last,
    Column::Average,
    Column::Best,
    Column::Worst,
    Column::StdDev,
    Column::Status,
    Column::Jitter,
    Column::Javg,
    Column::Jmax,
    Column::Jinta,
    Column::LastSrcPort,
    Column::LastDestPort,
    Column::LastSeq,
    Column::LastIcmpPacketType,
    Column::LastIcmpPacketCode,
    Column::LastNatStatus,
    Column::Failed,
    Column::Floss,
    Column::Bloss,
    Column::FlossPct,
    Column::Dscp,
    Column::Ecn,
    Column::Asn,
];

/// The columns shown by default (`--tui-custom-columns holsravbwdt`).
pub const DEFAULT: &str = "holsravbwdt";

impl Column {
    /// The one-letter code used by `--tui-custom-columns`.
    pub const fn code(self) -> char {
        match self {
            Self::Ttl => 'h',
            Self::Host => 'o',
            Self::LossPct => 'l',
            Self::Sent => 's',
            Self::Received => 'r',
            Self::Last => 'a',
            Self::Average => 'v',
            Self::Best => 'b',
            Self::Worst => 'w',
            Self::StdDev => 'd',
            Self::Status => 't',
            Self::Jitter => 'j',
            Self::Javg => 'g',
            Self::Jmax => 'x',
            Self::Jinta => 'i',
            Self::LastSrcPort => 'S',
            Self::LastDestPort => 'P',
            Self::LastSeq => 'Q',
            Self::LastIcmpPacketType => 'T',
            Self::LastIcmpPacketCode => 'C',
            Self::LastNatStatus => 'N',
            Self::Failed => 'f',
            Self::Floss => 'F',
            Self::Bloss => 'B',
            Self::FlossPct => 'D',
            Self::Dscp => 'K',
            Self::Ecn => 'M',
            Self::Asn => 'A',
        }
    }

    /// The table heading.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ttl => "#",
            Self::Host => "Host",
            Self::LossPct => "Loss%",
            Self::Sent => "Snd",
            Self::Received => "Recv",
            Self::Last => "Last",
            Self::Average => "Avg",
            Self::Best => "Best",
            Self::Worst => "Wrst",
            Self::StdDev => "StDev",
            Self::Status => "Sts",
            Self::Jitter => "Jttr",
            Self::Javg => "Javg",
            Self::Jmax => "Jmax",
            Self::Jinta => "Jint",
            Self::LastSrcPort => "Sprt",
            Self::LastDestPort => "Dprt",
            Self::LastSeq => "Seq",
            Self::LastIcmpPacketType => "Type",
            Self::LastIcmpPacketCode => "Code",
            Self::LastNatStatus => "Nat",
            Self::Failed => "Fail",
            Self::Floss => "Floss",
            Self::Bloss => "Bloss",
            Self::FlossPct => "Floss%",
            Self::Dscp => "Dscp",
            Self::Ecn => "Ecn",
            Self::Asn => "ASN",
        }
    }

    /// Parses a `--tui-custom-columns` code.
    pub fn from_code(code: char) -> Option<Self> {
        ALL.into_iter().find(|c| c.code() == code)
    }

    /// Whether this column is variable width (uses the remaining space).
    pub const fn is_variable(self) -> bool {
        matches!(self, Self::Host)
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// One column and whether it is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The column.
    pub column: Column,
    /// Whether it is displayed.
    pub shown: bool,
}

/// The columns of the hops table, shown ones first in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Columns(Vec<Entry>);

impl Default for Columns {
    fn default() -> Self {
        Self::from_codes(DEFAULT).expect("default columns")
    }
}

impl Columns {
    /// Builds the column set from a `--tui-custom-columns` string.
    ///
    /// The given columns are shown, in order; the remaining columns follow,
    /// hidden, in their natural order.
    pub fn from_codes(codes: &str) -> Result<Self, String> {
        let mut entries = Vec::with_capacity(ALL.len());
        let mut seen = Vec::new();
        for code in codes.chars() {
            let column =
                Column::from_code(code).ok_or_else(|| format!("unknown column code: {code}"))?;
            if seen.contains(&column) {
                return Err(format!("duplicate column: {column}"));
            }
            seen.push(column);
            entries.push(Entry {
                column,
                shown: true,
            });
        }
        if entries.is_empty() {
            return Err(String::from("at least one column is required"));
        }
        entries.extend(
            ALL.into_iter()
                .filter(|c| !seen.contains(c))
                .map(|column| Entry {
                    column,
                    shown: false,
                }),
        );
        Ok(Self(entries))
    }

    /// The shown columns, in display order.
    pub fn shown(&self) -> impl Iterator<Item = Column> + '_ {
        self.0.iter().filter(|e| e.shown).map(|e| e.column)
    }

    /// Every column, shown or not.
    pub fn all(&self) -> &[Entry] {
        &self.0
    }

    /// Toggles the visibility of the entry at `index`.
    pub fn toggle(&mut self, index: usize) {
        if let Some(entry) = self.0.get_mut(index) {
            entry.shown = !entry.shown;
        }
    }

    /// Moves the entry at `index` one place up.
    pub fn move_up(&mut self, index: usize) {
        if index > 0 && index < self.0.len() {
            self.0.swap(index - 1, index);
        }
    }

    /// Moves the entry at `index` one place down.
    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.0.len() {
            self.0.swap(index, index + 1);
        }
    }

    /// The current setting as a `--tui-custom-columns` string.
    pub fn to_codes(&self) -> String {
        self.shown().map(Column::code).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shows_the_expected_columns() {
        let columns = Columns::default();
        assert_eq!(columns.to_codes(), DEFAULT);
        assert_eq!(columns.shown().count(), 11);
        assert_eq!(columns.all().len(), ALL.len());
    }

    #[test]
    fn parsing_rejects_bad_input() {
        assert!(Columns::from_codes("hz").is_err());
        assert!(Columns::from_codes("hh").is_err());
        assert!(Columns::from_codes("").is_err());
    }

    #[test]
    fn toggle_and_reorder() {
        let mut columns = Columns::default();
        columns.toggle(0);
        assert_eq!(columns.shown().count(), 10);
        let first = columns.all()[0].column;
        columns.move_down(0);
        assert_eq!(columns.all()[1].column, first);
        columns.move_up(1);
        assert_eq!(columns.all()[0].column, first);
    }

    #[test]
    fn every_column_round_trips_through_its_code() {
        for column in ALL {
            assert_eq!(Column::from_code(column.code()), Some(column));
        }
    }
}
