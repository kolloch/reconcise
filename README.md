# Reconcise

* Reconcile your server config without state outside of your target platforms!
  Similarly, to e.g. the kustomize controller of flux CD, we do not rely on
  local state for syncing. Instead, we use labels to distinguish resources
  created by reconcise clearly.
* Use a rust library to configure your deployment.

STATUS: This is just some fun exploration. Not sure if this is heading anywhere.

## Concepts

Similarly to terraform, we use:

* **Resources** to represent things that can be created, destroyed, updated like
  virtual machines, volumes, ...
* **Providers** that actually know how to create, destroy, and/or update
  resources.
## Inspirations

* Terraform
* FluxCD
* Nix
