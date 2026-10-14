//! The view settings that can be changed while the window is open.

use crate::columns::Columns;
use crate::config::{AddressMode, AsMode, GeoIpMode, IcmpExtensionMode};
use crate::theme;

/// Everything the settings dialog can change.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    /// How addresses are rendered in the host column.
    pub address_mode: AddressMode,
    /// How autonomous system information is rendered.
    pub as_mode: AsMode,
    /// How ICMP extensions are rendered.
    pub icmp_extension_mode: IcmpExtensionMode,
    /// How GeoIP information is rendered.
    pub geoip_mode: GeoIpMode,
    /// Maximum addresses shown per hop; `None` shows all.
    pub max_addrs: Option<u8>,
    /// Hops up to this TTL are hidden; `None` shows all.
    pub privacy_max_ttl: Option<u8>,
    /// The columns of the hops table.
    pub columns: Columns,
    /// Whether to look up autonomous system information for hops.
    pub lookup_as_info: bool,
    /// The timezone of the clock, or `None` for the local one.
    pub timezone: Option<chrono_tz::Tz>,
    /// The theme.
    pub theme: theme::Settings,
    /// Whether the selected hop is shown with its details expanded.
    pub show_hop_details: bool,
}

impl Settings {
    /// The timezone of the clock, if one was configured.
    pub fn timezone(&self) -> Option<chrono_tz::Tz> {
        self.timezone
    }
}
