string_kind!(
    /// Semantic relationship between two graph nodes.
    EdgeKind,
    "edge",
    {
        Contains => "contains",
        Imports => "imports",
        Calls => "calls",
        References => "references",
        Method => "method",
        Implements => "implements",
        ReExports => "re_exports",
        DependsOn => "depends_on",
        Inherits => "inherits",
        Publishes => "publishes",
        Consumes => "consumes",
        Binds => "binds",
        Reads => "reads",
        Writes => "writes",
        Deploys => "deploys",
        Exposes => "exposes",
        Mounts => "mounts",
        Configures => "configures",
    }
);
