use bollard::query_parameters::CreateContainerOptions;
use bollard::Docker;
use rusty_console::application::container::{ContainerRepository, ContainerStatsEvent};
use rusty_console::application::image::ImageRepository;
use rusty_console::application::network::NetworkRepository;
use rusty_console::application::volume::VolumeRepository;
use rusty_console::infrastructure::docker::client::DockerClient;
use rusty_console::infrastructure::docker::container::ContainerAdapter;
use rusty_console::infrastructure::docker::image::ImageAdapter;
use rusty_console::infrastructure::docker::network::NetworkAdapter;
use rusty_console::infrastructure::docker::volume::VolumeAdapter;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::error::TryRecvError;

const ALPINE_IMAGE: &str = "alpine:3.20";
const BUSYBOX_IMAGE: &str = "busybox:latest";

fn unique_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    format!("rc-it-{}-{}", prefix, nanos)
}

async fn docker_available() -> bool {
    match Docker::connect_with_socket_defaults() {
        Ok(docker) => docker.ping().await.is_ok(),
        Err(_) => false,
    }
}

async fn setup() -> Option<(DockerClient, Docker)> {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping integration test");
        return None;
    }
    let client = DockerClient::new().expect("docker client");
    let bollard = Docker::connect_with_socket_defaults().expect("bollard client");
    Some((client, bollard))
}

async fn pull_image(bollard: &Docker, image: &str) {
    use bollard::query_parameters::CreateImageOptions;
    use futures_util::StreamExt;
    let options = CreateImageOptions {
        from_image: Some(image.to_string()),
        ..Default::default()
    };
    let mut stream = bollard.create_image(Some(options), None, None);
    while let Some(item) = stream.next().await {
        item.expect("image pull succeeded");
    }
}

async fn create_container(bollard: &Docker, name: &str, image: &str, cmd: Vec<&str>) -> String {
    pull_image(bollard, image).await;
    let config = bollard::models::ContainerCreateBody {
        image: Some(image.to_string()),
        cmd: Some(cmd.into_iter().map(String::from).collect()),
        ..Default::default()
    };
    let options = CreateContainerOptions {
        name: Some(name.to_string()),
        ..Default::default()
    };
    let result = bollard
        .create_container(Some(options), config)
        .await
        .expect("container created");
    result.id
}

async fn remove_container(bollard: &Docker, id: &str) {
    use bollard::query_parameters::RemoveContainerOptions;
    let _ = bollard
        .remove_container(
            id,
            Some(RemoveContainerOptions {
                force: true,
                v: true,
                ..Default::default()
            }),
        )
        .await;
}

#[tokio::test]
async fn container_lifecycle_integration() {
    let Some((client, bollard)) = setup().await else {
        return;
    };
    let adapter = ContainerAdapter::new(client);
    let name = unique_name("container");
    let id = create_container(&bollard, &name, ALPINE_IMAGE, vec!["sleep", "300"]).await;

    let found = adapter.get_all().await.expect("list containers");
    let docker_name = format!("/{}", name);
    let created = found
        .iter()
        .find(|c| c.name() == docker_name)
        .expect("created container listed");
    assert_eq!(created.id().as_str(), id);
    assert!(!created.is_running());

    adapter.start(&id).await.expect("start");
    let running = adapter.get_by_id(&id).await.expect("get container");
    assert_eq!(running.map(|c| c.is_running()), Some(true));

    adapter.pause(&id).await.expect("pause");
    adapter.unpause(&id).await.expect("unpause");

    let logs = adapter.get_logs(&id, Some(10)).await.expect("logs");
    assert!(logs.lines().count() <= 10);

    adapter.restart(&id).await.expect("restart");
    adapter.stop(&id).await.expect("stop");
    let stopped = adapter.get_by_id(&id).await.expect("get container");
    assert_eq!(stopped.map(|c| c.is_running()), Some(false));

    let missing = adapter
        .get_by_id("0000000000000000000000000000000000000000000000000000000000000000")
        .await;
    assert!(missing.is_err());

    adapter.delete(&id, false).await.expect("delete");
    let after = adapter.get_all().await.expect("list containers");
    assert!(after.iter().all(|c| c.name() != docker_name));
}

#[tokio::test]
async fn volume_lifecycle_integration() {
    let Some((client, bollard)) = setup().await else {
        return;
    };
    let adapter = VolumeAdapter::new(client);
    let name = unique_name("volume");

    bollard
        .create_volume(bollard::models::VolumeCreateRequest {
            name: Some(name.clone()),
            ..Default::default()
        })
        .await
        .expect("volume created");

    let all = adapter.get_all().await.expect("list volumes");
    let volume = all
        .iter()
        .find(|v| v.name() == name)
        .expect("volume listed");
    assert_eq!(volume.driver(), "local");
    assert!(!volume.in_use());

    let by_name = adapter.get_by_name(&name).await.expect("get_by_name");
    assert!(by_name.is_some());

    adapter.delete(&name).await.expect("delete volume");
    let after = adapter
        .get_by_name(&name)
        .await
        .expect("get_by_name after delete");
    assert!(after.is_none());
}

#[tokio::test]
async fn network_lifecycle_integration() {
    let Some((client, bollard)) = setup().await else {
        return;
    };
    let adapter = NetworkAdapter::new(client);
    let name = unique_name("network");

    bollard
        .create_network(bollard::models::NetworkCreateRequest {
            name: name.clone(),
            ..Default::default()
        })
        .await
        .expect("network created");

    let all = adapter.get_all().await.expect("list networks");
    let network = all
        .iter()
        .find(|n| n.name() == name)
        .expect("network listed");
    assert_eq!(network.driver(), "bridge");

    let by_name = adapter.get_by_name(&name).await.expect("get_by_name");
    assert!(by_name.is_some());

    let inspected = adapter.inspect(&name).await.expect("inspect");
    assert_eq!(inspected.name(), name);

    let container_name = unique_name("net-container");
    let id = create_container(&bollard, &container_name, ALPINE_IMAGE, vec!["sleep", "60"]).await;
    bollard
        .connect_network(
            &name,
            bollard::models::NetworkConnectRequest {
                container: id.clone(),
                ..Default::default()
            },
        )
        .await
        .expect("container connected to network");

    adapter
        .disconnect(&name, &id, false)
        .await
        .expect("disconnect");

    remove_container(&bollard, &id).await;
    adapter.delete(&name).await.expect("delete network");
    let after = adapter
        .get_by_name(&name)
        .await
        .expect("get_by_name after delete");
    assert!(after.is_none());
}

#[tokio::test]
async fn image_lifecycle_integration() {
    let Some((client, bollard)) = setup().await else {
        return;
    };
    let adapter = ImageAdapter::new(client);

    pull_image(&bollard, ALPINE_IMAGE).await;

    let all = adapter.get_all().await.expect("list images");
    assert!(all.iter().any(|i| i.repository() == "alpine"));

    let alpine = all
        .iter()
        .find(|i| i.repository() == "alpine" && i.tag() == "3.20")
        .expect("alpine image present");
    let by_id = adapter
        .get_by_id(alpine.id().as_str())
        .await
        .expect("get image by id");
    assert!(by_id.is_some());

    pull_image(&bollard, BUSYBOX_IMAGE).await;
    let inspect = bollard
        .inspect_image(BUSYBOX_IMAGE)
        .await
        .expect("busybox image present");
    let busybox_id = inspect.id.expect("image id");

    let deleted = adapter.delete(&busybox_id, false).await;
    if deleted.is_ok() {
        let after = adapter
            .get_by_id(&busybox_id)
            .await
            .expect("get busybox by id");
        assert!(after.is_none());
    } else {
        eprintln!("busybox image in use, delete skipped");
    }
}

#[tokio::test]
async fn container_stats_subscription_integration() {
    let Some((client, bollard)) = setup().await else {
        return;
    };
    let adapter = ContainerAdapter::new(client);
    let name = unique_name("stats");
    let id = create_container(&bollard, &name, ALPINE_IMAGE, vec!["sleep", "300"]).await;
    adapter.start(&id).await.expect("start");

    let subscription = adapter
        .subscribe_stats(vec![id.clone()])
        .await
        .expect("subscribe stats");
    let mut subscription = subscription;
    let event = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match subscription.try_recv() {
                Ok(event) => break event,
                Err(TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
                Err(TryRecvError::Disconnected) => panic!("stats channel closed"),
            }
        }
    })
    .await
    .expect("stats event received within timeout");

    match event {
        ContainerStatsEvent::Update(update) => assert_eq!(update.container_id, id),
        ContainerStatsEvent::Error {
            container_id,
            message,
        } => {
            panic!("unexpected stats error for {}: {}", container_id, message)
        }
    }

    remove_container(&bollard, &id).await;
}

#[tokio::test]
async fn volume_in_use_detection_integration() {
    let Some((client, bollard)) = setup().await else {
        return;
    };
    let adapter = VolumeAdapter::new(client);
    let name = unique_name("inuse");

    bollard
        .create_volume(bollard::models::VolumeCreateRequest {
            name: Some(name.clone()),
            ..Default::default()
        })
        .await
        .expect("volume created");

    let container_name = unique_name("vol-container");
    let config = bollard::models::ContainerCreateBody {
        image: Some(ALPINE_IMAGE.to_string()),
        cmd: Some(vec!["sleep".to_string(), "60".to_string()]),
        host_config: Some(bollard::models::HostConfig {
            binds: Some(vec![format!("{}:/data", name)]),
            ..Default::default()
        }),
        ..Default::default()
    };
    let options = CreateContainerOptions {
        name: Some(container_name.clone()),
        ..Default::default()
    };
    let created = bollard
        .create_container(Some(options), config)
        .await
        .expect("container created");

    let all = adapter.get_all().await.expect("list volumes");
    let volume = all
        .iter()
        .find(|v| v.name() == name)
        .expect("volume listed");
    assert!(volume.in_use());
    assert!(volume
        .linked_containers()
        .contains(&format!("/{}", container_name)));

    remove_container(&bollard, &created.id).await;
    adapter
        .delete(&name)
        .await
        .expect("volume deleted after container removed");
}
