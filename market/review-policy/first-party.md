# First-party package review policy

First-party packages must provide:

- manifest schema validation;
- explicit `implementationStatus`; `planned` packages claim no components and `development` is not runtime completion;
- exact `FirstPartySystemPlugin` default-install/default-enable policy with independent user disable;
- explicit capability list;
- default manifests use only registered auto-grant-eligible public read/link-out capabilities;
- source authority policy;
- explicit tenant-private data minimization statement, including `none` when no private data is in scope;
- acceptance matrix bindings;
- disabling/revocation behavior;
- no raw credentials and no hidden network mutation.
