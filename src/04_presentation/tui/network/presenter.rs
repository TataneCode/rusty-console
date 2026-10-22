use crate::application::network::NetworkDto;
use crate::presentation::tui::common::{apply_name_sort, FilterState, SortOrder, TableSelection};

pub struct NetworkPresenter {
    pub networks: Vec<NetworkDto>,
    pub selection: TableSelection,
    pub selected_network: Option<NetworkDto>,
    pub loading: bool,
    pub filter: FilterState,
    pub sort_order: SortOrder,
    pub container_selection: TableSelection,
}

pub fn filter_networks<'items>(
    networks: &'items [NetworkDto],
    filter: &str,
) -> Vec<&'items NetworkDto> {
    if filter.is_empty() {
        networks.iter().collect()
    } else {
        let filter_lower = filter.to_lowercase();
        networks
            .iter()
            .filter(|network| network.name.to_lowercase().contains(&filter_lower))
            .collect()
    }
}

impl NetworkPresenter {
    pub fn new() -> Self {
        NetworkPresenter {
            networks: Vec::new(),
            selection: TableSelection::new(),
            selected_network: None,
            loading: false,
            filter: FilterState::new(),
            sort_order: SortOrder::default(),
            container_selection: TableSelection::new(),
        }
    }

    pub fn set_networks(&mut self, networks: Vec<NetworkDto>) {
        self.networks = networks;
        self.update_filtered_selection();
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn filtered_networks(&self) -> Vec<&NetworkDto> {
        let mut filtered = filter_networks(&self.networks, self.filter.value());
        apply_name_sort(&mut filtered, self.sort_order, |network| {
            network.name.as_str()
        });
        filtered
    }
    pub fn toggle_sort(&mut self) {
        self.sort_order = self.sort_order.next();
        self.update_filtered_selection();
    }

    pub fn selected_network(&self) -> Option<&NetworkDto> {
        let filtered = self.filtered_networks();
        self.selection
            .selected()
            .and_then(|i| filtered.get(i).copied())
    }

    pub fn set_details(&mut self, network: NetworkDto) {
        self.container_selection.set_items(network.containers.len());
        self.selected_network = Some(network);
    }

    pub fn clear_details(&mut self) {
        self.selected_network = None;
        self.container_selection = TableSelection::new();
    }

    pub fn select_network_by_name(&mut self, name: &str) {
        if let Some(network) = self
            .networks
            .iter()
            .find(|network| network.name == name)
            .cloned()
        {
            self.set_details(network);
        }
    }

    pub fn selected_network_container(
        &self,
    ) -> Option<&crate::application::network::NetworkContainerDto> {
        self.selected_network.as_ref().and_then(|network| {
            self.container_selection
                .selected()
                .and_then(|i| network.containers.get(i))
        })
    }

    pub fn navigate_up(&mut self) {
        self.selection.previous();
    }

    pub fn navigate_down(&mut self) {
        self.selection.next();
    }

    pub fn navigate_container_up(&mut self) {
        self.container_selection.previous();
    }

    pub fn navigate_container_down(&mut self) {
        self.container_selection.next();
    }

    pub fn activate_filter(&mut self) {
        self.filter.activate();
    }

    pub fn deactivate_filter(&mut self) {
        self.filter.deactivate();
        self.update_filtered_selection();
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push_char(c);
        self.update_filtered_selection();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop_char();
        self.update_filtered_selection();
    }

    pub fn is_filter_active(&self) -> bool {
        self.filter.is_active()
    }

    pub fn active_filter(&self) -> Option<&str> {
        self.filter.active_value()
    }

    fn update_filtered_selection(&mut self) {
        let count = self.filtered_networks().len();
        self.selection.set_items(count);
    }
}

impl Default for NetworkPresenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::network::NetworkContainerDto;

    fn make_network(name: &str, containers: usize) -> NetworkDto {
        NetworkDto {
            id: format!("id_{name}"),
            name: name.to_string(),
            driver: "bridge".to_string(),
            scope: "local".to_string(),
            internal: false,
            attachable: false,
            subnet: Some("172.20.0.0/16".to_string()),
            gateway: Some("172.20.0.1".to_string()),
            created: "2024-01-01".to_string(),
            builtin: false,
            containers: (0..containers)
                .map(|index| NetworkContainerDto {
                    name: format!("container-{index}"),
                    ip: format!("172.20.0.{index}"),
                })
                .collect(),
            can_delete: containers == 0,
            container_count: containers,
        }
    }

    fn three_networks() -> Vec<NetworkDto> {
        vec![
            make_network("app-net", 0),
            make_network("db-net", 2),
            make_network("web-net", 1),
        ]
    }

    #[test]
    fn test_set_networks_updates_list() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        assert_eq!(p.networks.len(), 3);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_filtered_networks_by_name() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        // Case-insensitive: "DB" should match "db-net"
        for c in "DB".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_networks();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "db-net");
    }

    #[test]
    fn test_selected_network_from_filtered_list() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        for c in "web".chars() {
            p.push_filter_char(c);
        }
        let selected = p.selected_network();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "web-net");
    }

    #[test]
    fn test_set_details_resets_container_selection() {
        let mut p = NetworkPresenter::new();
        p.set_details(make_network("db-net", 3));
        assert!(p.selected_network.is_some());
        assert_eq!(p.container_selection.selected(), Some(0));
        assert_eq!(p.container_selection.items_count, 3);

        p.navigate_container_down();
        assert_eq!(p.container_selection.selected(), Some(1));

        p.set_details(make_network("db-net", 1));
        assert_eq!(p.container_selection.selected(), Some(0));
    }

    #[test]
    fn test_selected_network_container() {
        let mut p = NetworkPresenter::new();
        p.set_details(make_network("db-net", 2));
        p.navigate_container_down();

        let container = p.selected_network_container().unwrap();
        assert_eq!(container.name, "container-1");
        assert_eq!(container.ip, "172.20.0.1");
    }

    #[test]
    fn test_clear_details_resets_container_selection() {
        let mut p = NetworkPresenter::new();
        p.set_details(make_network("db-net", 2));
        p.clear_details();
        assert!(p.selected_network.is_none());
        assert!(p.selected_network_container().is_none());
        assert_eq!(p.container_selection.items_count, 0);
    }

    #[test]
    fn test_select_network_by_name_refreshes_details() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        p.select_network_by_name("db-net");
        assert_eq!(
            p.selected_network.as_ref().map(|n| n.name.as_str()),
            Some("db-net")
        );

        p.select_network_by_name("missing");
        // Details unchanged when the name is absent
        assert_eq!(
            p.selected_network.as_ref().map(|n| n.name.as_str()),
            Some("db-net")
        );
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        p.activate_filter();
        assert!(p.filter.is_active());
        p.push_filter_char('x');
        p.deactivate_filter();
        assert!(!p.filter.is_active());
        assert!(p.filter.value().is_empty());
        // All items visible again
        assert_eq!(p.filtered_networks().len(), 3);
    }

    #[test]
    fn test_toggle_sort_cycles_orders() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_networks()
            .iter()
            .map(|n| n.name.as_str())
            .collect();
        assert_eq!(names, vec!["app-net", "db-net", "web-net"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_networks()
            .iter()
            .map(|n| n.name.as_str())
            .collect();
        assert_eq!(names, vec!["web-net", "db-net", "app-net"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_networks()
            .iter()
            .map(|n| n.name.as_str())
            .collect();
        assert_eq!(names, vec!["app-net", "db-net", "web-net"]);
    }

    #[test]
    fn test_filter_narrows_selection() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        p.navigate_down();
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(2));
        // Filter to 1 item
        for c in "app-net".chars() {
            p.push_filter_char(c);
        }
        assert_eq!(p.filtered_networks().len(), 1);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_navigate_wraps() {
        let mut p = NetworkPresenter::new();
        p.set_networks(three_networks());
        assert_eq!(p.selection.selected(), Some(0));
        // Wrap backwards from 0
        p.navigate_up();
        assert_eq!(p.selection.selected(), Some(2));
        // Wrap forwards from last
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_set_loading_changes_state() {
        let mut p = NetworkPresenter::new();
        assert!(!p.is_loading());

        p.set_loading(true);
        assert!(p.is_loading());

        p.set_loading(false);
        assert!(!p.is_loading());
    }
}
