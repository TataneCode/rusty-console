use crate::domain::stack::{Stack, StackContainer, StackContainerState, StackName, STANDALONE};
use bollard::models::{ContainerSummary, PortSummary};
use std::collections::HashMap;

pub struct StackInfraMapper;

impl StackInfraMapper {
    /// Groups a flat list of ContainerSummary into Stacks by the
    /// `com.docker.compose.project` label. Containers without the label
    /// are grouped under STANDALONE.
    pub fn group_into_stacks(summaries: Vec<ContainerSummary>) -> Vec<Stack> {
        let mut groups: HashMap<String, Vec<ContainerSummary>> = HashMap::new();

        for summary in summaries {
            let key = summary
                .labels
                .as_ref()
                .and_then(|l| l.get("com.docker.compose.project"))
                .cloned()
                .unwrap_or_else(|| STANDALONE.to_string());
            groups.entry(key).or_default().push(summary);
        }

        let mut stacks: Vec<Stack> = groups
            .into_iter()
            .filter_map(|(name, summaries)| {
                let stack_name = StackName::new(&name).ok()?;
                let containers = summaries.iter().filter_map(Self::map_container).collect();
                Some(Stack::new(stack_name, containers))
            })
            .collect();

        // Deterministic order: alphabetical, standalone last
        stacks.sort_by(|a, b| {
            let a_standalone = a.name().is_standalone();
            let b_standalone = b.name().is_standalone();
            match (a_standalone, b_standalone) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => a.name().as_str().cmp(b.name().as_str()),
            }
        });

        stacks
    }

    fn map_container(summary: &ContainerSummary) -> Option<StackContainer> {
        let id = summary.id.as_ref()?.clone();
        let name = summary
            .names
            .as_ref()
            .and_then(|n| n.first())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let image = summary
            .image
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let state = summary
            .state
            .as_ref()
            .map(|s| {
                s.to_string()
                    .parse::<StackContainerState>()
                    .unwrap_or(StackContainerState::Stopped)
            })
            .unwrap_or(StackContainerState::Stopped);
        let status = summary.status.clone().unwrap_or_default();
        let ports = Self::map_ports(summary.ports.as_ref());

        Some(StackContainer::new(id, name, image, state, status, ports))
    }

    fn map_ports(ports: Option<&Vec<PortSummary>>) -> String {
        let mapped = ports
            .map(|ports| {
                ports
                    .iter()
                    .map(|port| {
                        let protocol = port
                            .typ
                            .as_ref()
                            .map(|typ| format!("{:?}", typ).to_lowercase())
                            .unwrap_or_else(|| "tcp".to_string());
                        match port.public_port {
                            Some(public) => {
                                format!("{}:{}/{}", public, port.private_port, protocol)
                            }
                            None => format!("{}/{}", port.private_port, protocol),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if mapped.is_empty() {
            "-".to_string()
        } else {
            mapped.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{ContainerSummaryStateEnum, PortSummary, PortSummaryTypeEnum};

    fn summary(id: &str, name: &str, stack: Option<&str>) -> ContainerSummary {
        let labels = stack.map(|stack_name| {
            HashMap::from([(
                "com.docker.compose.project".to_string(),
                stack_name.to_string(),
            )])
        });

        ContainerSummary {
            id: Some(id.to_string()),
            names: Some(vec![format!("/{name}")]),
            image: Some("nginx:latest".to_string()),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            status: Some("Up 5 minutes".to_string()),
            labels,
            ..Default::default()
        }
    }

    #[test]
    fn groups_compose_containers_into_named_stacks() {
        let stacks = StackInfraMapper::group_into_stacks(vec![
            summary("a1", "web", Some("app")),
            summary("a2", "worker", Some("app")),
            summary("b1", "db", Some("data")),
        ]);

        assert_eq!(stacks.len(), 2);
        assert_eq!(stacks[0].name().as_str(), "app");
        assert_eq!(stacks[0].container_count(), 2);
        assert_eq!(stacks[0].container_ids(), vec!["a1", "a2"]);
        assert_eq!(stacks[1].name().as_str(), "data");
        assert_eq!(stacks[1].container_count(), 1);
    }

    #[test]
    fn groups_missing_compose_labels_under_standalone() {
        let stacks = StackInfraMapper::group_into_stacks(vec![
            summary("a1", "web", None),
            summary("a2", "worker", Some("app")),
        ]);

        assert_eq!(stacks.len(), 2);
        assert_eq!(stacks[0].name().as_str(), "app");
        assert!(stacks[1].name().is_standalone());
        assert_eq!(stacks[1].container_count(), 1);
        assert_eq!(stacks[1].container_ids(), vec!["a1"]);
    }

    #[test]
    fn sorts_stacks_alphabetically_with_standalone_last() {
        let stacks = StackInfraMapper::group_into_stacks(vec![
            summary("z1", "web", Some("zeta")),
            summary("a1", "api", Some("alpha")),
            summary("s1", "misc", None),
            summary("b1", "db", Some("beta")),
        ]);

        let names: Vec<&str> = stacks.iter().map(|stack| stack.name().as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta", "zeta", STANDALONE]);
    }

    #[test]
    fn skips_invalid_container_summaries_when_building_stacks() {
        let invalid = ContainerSummary {
            id: None,
            names: Some(vec!["/broken".to_string()]),
            labels: Some(HashMap::from([(
                "com.docker.compose.project".to_string(),
                "app".to_string(),
            )])),
            ..Default::default()
        };

        let stacks =
            StackInfraMapper::group_into_stacks(vec![summary("a1", "web", Some("app")), invalid]);

        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].name().as_str(), "app");
        assert_eq!(stacks[0].container_count(), 1);
        assert_eq!(stacks[0].containers()[0].display_name(), "web");
    }

    #[test]
    fn formats_container_ports_for_stack_container_display() {
        let mut web = summary("a1", "web", Some("app"));
        web.ports = Some(vec![
            PortSummary {
                private_port: 80,
                public_port: Some(8080),
                typ: Some(PortSummaryTypeEnum::TCP),
                ..Default::default()
            },
            PortSummary {
                private_port: 53,
                public_port: Some(5353),
                typ: Some(PortSummaryTypeEnum::UDP),
                ..Default::default()
            },
            PortSummary {
                private_port: 443,
                public_port: None,
                typ: Some(PortSummaryTypeEnum::TCP),
                ..Default::default()
            },
        ]);

        let stacks = StackInfraMapper::group_into_stacks(vec![web]);
        let container = &stacks[0].containers()[0];

        assert_eq!(container.ports(), "8080:80/tcp, 5353:53/udp, 443/tcp");
    }

    #[test]
    fn uses_dash_when_container_has_no_ports() {
        let stacks = StackInfraMapper::group_into_stacks(vec![summary("a1", "web", Some("app"))]);
        assert_eq!(stacks[0].containers()[0].ports(), "-");
    }
}
