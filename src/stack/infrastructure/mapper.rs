use crate::container::infrastructure::mapper::ContainerInfraMapper;
use crate::stack::domain::{Stack, StackName, STANDALONE};
use bollard::models::ContainerSummary;
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
                let containers = summaries
                    .iter()
                    .filter_map(ContainerInfraMapper::from_docker)
                    .collect();
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
}
