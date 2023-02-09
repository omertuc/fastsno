. as $root 
| [
    {
        "group": "audit",
        "data": [
            $root[]
            | select(
                .verb == "create"
                and (.objectRef.resource | test("(pods|deployments|secrets|replicasets|configmaps|namespaces)"))
                and .responseStatus.code == 201
                and .objectRef.name != null
            ) 
            | {
                "label": "\(.objectRef.namespace) - \(.objectRef.name) - \(.objectRef.resource)",
                "data": [
                    {
                        "timeRange": [ .requestReceivedTimestamp, $root[-1].requestReceivedTimestamp ],
                        "val": .objectRef.resource
                    }
                ]
            }
        ]
    }
]
