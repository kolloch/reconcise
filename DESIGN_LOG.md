### Prefer references over names for linking (2021-05-09)

Everything has a name but if we use the names to link between resources or
resources and providers and the like, we get errors at runtime that could be
compile errors. We try to avoid that where easily possible. A high-level
use of the API might look like this:

```rust
let mut hcloud_provider = HCloudProvider.builder(
  HCloudProviderConfig {
    // Some labels to detect resources that were created by this
    // config before. If you have multiple separate configs managed by reconcise,
    // you need to make sure to use different labels.
    uniqueLabels: map![
      "managed_by" -> "reconcise"
    ],
    // ...
  }
);

let mut hcloud_k3s_provider = HCloudK3sProvider.builder(
  HCloudK3sProvider {
    hcloud_provider.clone(), // do we always use arcs for providers or the like?
    // ...
  }
);

// RuntimeVar must be some sort of future but it would be nice to make dependencies
// inspectable. It is hard to write a type-safe API for this if we allow
// a variable number of outputs as inputs.
let k8s_cluster_output: RuntimeVar<K8sCluster> = hcloud_k3s_provider.add_k3s_cluster(
  HCloudK3sClusterSpec {
    api_nodes: 3,
    api_server_type: "CX3".to_string(),
    agent_nodes: 10,
    agent_node_type: "CX25".to_string(),
    // ...
  }
)

struct K8sCluster {
  spec: Arc<HCloudK3sClusterSpec>,
  output: Arc<HCloudK3sClusterOutput>,
}

// Open design decision:
//
// If we allow the provider config to depend on runtime vars,
// we might also want to delay adding of resources.
// That "hides" these resources from the original plan but sometimes
// it is not even known which number of resources etc we want to build
// at this stage.
let mut k8s_provider = K8sProvider.builder_with_runtime_var(
  k8s_cluster_output.map( |out| {
    // Map to right shape to decouple providers...
    K8sConfig {
      // Some labels to detect resources that were created by this
      // config before. If you have multiple separate configs managed by reconcise,
      // you need to make sure to use different labels.
      uniqueLabels: map![
        "managed_by" -> "reconcise"
      ],
      api_ips: out.k8s_api_ips.clone(),
      // ...
    }
  });
)

// Also returns some output but it is unused.
k8s_provider.add_deployment(
  K8sDeployment {
    metadata: K8sMetadata {
      name: "xyz",
    },
    spec: K8sDeploymentSpec {
      // ...
    },
  }
);

// Maybe we also want to allow to wait for stuff that we didn't create?
let service: RuntimeVar<K8sService> =
  k8s_provider.wait_for_external_ip_of_existing_service("some service", "some namespace");

let mut cloud_flare_provider = CloudFlareProvider.builder(
  CloudFlareProviderConfig {
    // ...
  }
);

// Add a dynamic resource which depends on a runtime var.
//
// We might at least require the name to be available before the runtime var is resolved.
//
// In a simple model, we would only sync resources, after all "dynamic" resources have
// been resolved. This is a simple predictable model but makes one runtime var failure
// block everything.
//
// Potentially, you could have something like `cloud_flare_provider.add_scope()` which in
// turn has all the build methods but will sync independently of other "scopes".
cloud_flare_provider.add_dns_record_with_runtime_var(
  DnsRecord {
    // ...
  }
)

// We only need to list providers here, because they know which resources where
// added to them.
let model = build_model(vec![
  hcloud_provider,
  hcloud_k3s_provider,
  k8s_provider,
  cloud_flare_provider,
]);

// Various methods, e.g. show the model without executing it, executing it...
model.show();
model.reconcile();
```


### Everything has a name (2021-05-09)

Every resource, provider should have a name for easy debuggability. The name is
"namespaced" by provider and resource type. It can be auto-detected from the
resource config. E.g. if a hetzner server has a name anyways, this can be used
as the "resource name".

Downside: They must be unique which can probably only be checked at runtime. The
important thing is that this can be checked early on and it is easy to resolve.

### Declarative vs imperative (2021-05-09)

I clearly want declarative building blocks. The problem might necessitate a
certain dynamic approach, probably not all can be handled with types. Therefore,
execution of the model needs to be strictly separated from creating the model
to enable pre-verification and inspection.

### Do provider handle multipe resources or not? (2021-05-09)

On the one hand, providers need to be created in code and often configured (e.g.
with an API key, ...). The less the user has to setup, the better. In terraform,
there are e.g. providers for "hetzner cloud" or "google cloud" which can then
handle a myriad of resources and only need to be configured once.

On the other hand, the code might get more complicated if we have a one-to-many
relationship between providers and resources. So it might be adventageous to
have a single resource provider.

Decision: It is nice to stick with the user friendly approach and ultimately go
with one provider. It is OK to have an intermediate entity, maybe called
ResourceBinding to simplify the code.
