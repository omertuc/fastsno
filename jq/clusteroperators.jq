def is_interesting_condition:
    .type == "Available" or
    .type == "Degraded";

def summarize_condition:
    if .type == "Available" and .status == "True" then
        "Available"
    elif .type == "Available" and .status == "False" then
        "Unavailable"
    elif .type == "Degraded" and .status == "True" then
        "Degraded"
    elif .type == "Degraded" and .status == "False" then
        "Undegraded"
    else
        "\(.type)\(.status)"
    end;

def summarize_conditions:
    [.[] | summarize_condition] | join("/");

def summarize_event_conditions:
    [.requestObject.status.conditions[]? | select(is_interesting_condition)] | sort_by(.type) | summarize_conditions;

def remove_repetition_by(key):
    reduce .[] as $item (
        [];
        if (.[-1] | key) == ($item | key) then
            (.)
        else 
            (. + [$item])
        end
    );

# Collect all resource names
[
    .[] | select(.objectRef.resource == "clusteroperators" and (.verb == "update" or .verb == "create") and (.requestObject.status.conditions != null))
] as $all_events 
| [$all_events[].requestObject.metadata.name]
| unique
as $resource_names

# Generate timeline data
| [
    {
        "group": "clusteroperators",
        "data": [
            $resource_names[]
            | {
                "label": .,
                "data": [
                    # Filter all events to find only those updating the current resource
                    [
                        . as $resource_name
                        | $all_events[]
                        | select(.requestObject.metadata.name == $resource_name)
                        # Remove events without request
                        | select(.requestObject != null)
                    ]

                    | sort_by(.requestReceivedTimestamp)

                    # Remove uninteresting updates that didn't change the conditions we care about
                    # | remove_repetition_by(summarize_event_conditions)

                    # Combine timestampts
                    | [
                        .[]
                    ] as $events
                    | [$events, $events[1:]]
                    | transpose[]
                    | {
                        "val": .[0] | summarize_event_conditions,
                        "timeRange": [
                            .[0].requestReceivedTimestamp,
                            (.[1].requestReceivedTimestamp // $all_events[-1].requestReceivedTimestamp)
                        ],
                        "description": [
                            [.[0] | .requestObject.status.conditions[] | "\(.type) - \(.status) - \(.message)"] | join("</br>")
                        ],
                    }
                ]
             }
        ]
    }
]


