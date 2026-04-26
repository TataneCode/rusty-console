use crate::stack::domain::{Stack, StackContainer, StackContainerState, StackName, STANDALONE};
use bollard::models::{ContainerSummary, Port};
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
            .and_then(|s| s.parse().ok())
            .unwrap_or(StackContainerState::Stopped);
        let status = summary.status.clone().unwrap_or_default();
        let ports = Self::map_ports(summary.ports.as_ref());

        Some(StackContainer::new(id, name, image, state, status, ports))
    }

    fn map_ports(ports: Option<&Vec<Port>>) -> String {
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
