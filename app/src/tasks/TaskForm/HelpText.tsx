import { NewTask } from "../../ApiClient";
import { Alert, FormText, FormLabel } from "react-bootstrap";
import { Field } from ".";
import React from "react";
import { OutLink } from "../../util";

export const HELPS: {
  [K in Field<NewTask>]?: {
    title: string;
    short: React.ReactNode;
    long?: React.ReactNode;
  };
} = {
  name: {
    title: "Task Name",
    short:
      "A short name to identify this task in this application. This can be edited.",
  },

  leader_aggregator_id: {
    title: "Leader Aggregator",
    short: "Select an aggregator server to process this metrics task.",
    long: (
      <p>
        The leader aggregator is one of the two non-colluding servers that
        processes metrics tasks. Its role is more resource-intensive than the
        helper&apos;s. One of the two aggregators must be run by Divvi Up, and
        the other must be run by a different organization. To use a self-hosted
        aggregator, you must first add it to your account; it will then appear
        in this list.
      </p>
    ),
  },

  helper_aggregator_id: {
    title: "Helper Aggregator",
    short: "Select an aggregator server to process this metrics task.",
    long: (
      <p>
        The helper aggregator is one of the two non-colluding servers that
        processes metrics tasks. Its role is less resource-intensive than the
        leader&apos;s. One of the two aggregators must be run by Divvi Up, and
        the other must be run by a different organization. To use a self-hosted
        aggregator, you must first add it to your account; it will then appear
        in this list.
      </p>
    ),
  },

  min_batch_size: {
    title: "Minimum Batch Size",
    short: "Minimum number of reports per batch.",
    long: (
      <p>
        This number should be set high enough that the aggregate results over a
        batch do not violate the application&apos;s privacy goals. This is
        determined by a number of factors, including the aggregation function
        used, the population distribution of measurements, the
        importance/sensitivity of the underlying data, and whether client
        attestation is used to prevent{" "}
        <OutLink href="https://www.ietf.org/archive/id/draft-ietf-ppm-dap-05.html#section-7-4.3">
          Sybil attacks
        </OutLink>
        . If differential privacy noise is added, it can simplify selection of a
        minimum batch size.
      </p>
    ),
  },

  "vdaf.type": {
    title: "Function",
    short:
      "Determines the kind of client measurement accepted, and how they are summarized.",
    long: (
      <>
        <p>Selects the aggregation function used by this metrics task.</p>
        <p>The following functions are supported:</p>
        <ul>
          <li>
            Count: Each client measurement is either "true" or "false". The
            aggregate result is the number of "true" measurements.
          </li>
          <li>
            Sum: Each client measurement is an integer number. The aggregate
            result is the sum of the measurements.
          </li>
          <li>
            Histogram: The aggregate result is a list of counters, and each
            client measurement chooses one counter to increment.
          </li>
        </ul>
      </>
    ),
  },

  "vdaf.bits": {
    title: "Measurement Range",
    short: "Selects the bit width and range of valid client measurements.",
    long: (
      <p>
        Determines the range of integers that are accepted as client
        measurements. Note that this only determines the maximum value of
        individual measurements, and not the maximum value of the aggregate
        result (sum of measurements). Regardless of this choice, the aggregate
        result wraps around at about 3.4×10<sup>38</sup>. This parameter affects
        the size of client reports.
      </p>
    ),
  },

  "vdaf.buckets": {
    title: "Histogram",
    short: "Specifies the boundaries between histogram buckets.",
    long: (
      <p>
        A comma-separated list of numbers, in ascending order. Each client
        measurement will only record whether it is below all bucket boundaries,
        between a particular pair of bucket boundaries, or above all bucket
        boundaries. Each bucket interval includes its upper boundary value, and
        excludes its lower boundary value. This parameter affects the size of
        client reports.
      </p>
    ),
  },

  max_batch_size: {
    title: "Query Type",
    short:
      "Determines how reports are grouped into batches, and what kinds of queries the collector can make.",
    long: (
      <>
        <h5>Time Interval:</h5>
        <p>
          Groups measurements into batches by their client timestamp. Collectors
          may query for aggregate results over (non-overlapping) time intervals.
          Good for identifying temporal patterns in data. If client reports may
          be received late, well after their timestamps, then the collector is
          forced to choose between delaying collection requests or abandoning
          late reports.
        </p>
        <h5>Fixed Size:</h5>
        <p>
          Groups measurements into batches arbitrarily as they arrive. Grants
          more control over batch sizes, because a maximum batch size can be
          set. Good for cases where the report upload rate is unknown or varies
          widely. Temporal patterns in data may be obscured by aggregating
          on-time and late reports together in the same batches.
        </p>
      </>
    ),
  },

  collector_credential_id: {
    title: "Collector Credentials",
    short: "Credentials that can be used to retrieve aggregates for this task.",
    long: (
      <p>
        Select a previously uploaded collector credential. You will need to use
        the credential&apos;s corresponding private key and auth token when
        collecting aggregate results.
      </p>
    ),
  },

  time_precision_seconds: {
    title: "Time precision",
    short: "Granularity of client report timestamps.",
    long: (
      <>
        <p>
          All client report timestamps will be rounded down to the previous
          multiple of this duration.
        </p>
        <p>
          If the query type is Time Interval, then query time intervals must
          have a duration at least as long as the time precision, must have a
          duration that is a multiple of the time precision, and must start and
          end on multiples of the time precision as well.
        </p>
      </>
    ),
  },
};

export function LongHelpText({ field }: { field: Field<NewTask> }) {
  const help = HELPS[field];
  if (help && help.long) {
    return (
      <Alert>
        <Alert.Heading>{help.title}</Alert.Heading>
        {help.long}
      </Alert>
    );
  } else {
    return null;
  }
}

export function ShortHelpText({
  fieldKey,
  setFocusedField,
}: {
  fieldKey: Field<NewTask>;
  setFocusedField(field: Field<NewTask>): void;
}) {
  const expand = React.useCallback(
    (e: React.MouseEvent) => {
      setFocusedField(fieldKey);
      e.preventDefault();
    },
    [setFocusedField, fieldKey],
  );

  const help = HELPS[fieldKey];
  if (help) {
    return (
      <div>
        <FormText muted>
          {help.short}
          {help.long ? (
            <>
              {" "}
              <a onClick={expand} href="#">
                more&raquo;
              </a>
            </>
          ) : null}
        </FormText>
      </div>
    );
  } else {
    return null;
  }
}

export function ShortHelpAndLabel({
  htmlFor,
  fieldKey,
  setFocusedField,
}: {
  htmlFor?: string;
  fieldKey: Field<NewTask>;
  setFocusedField(field: Field<NewTask>): void;
}) {
  const help = HELPS[fieldKey];
  if (help && help.title) {
    return (
      <>
        <FormLabel htmlFor={htmlFor}>{help.title}</FormLabel>
        <ShortHelpText fieldKey={fieldKey} setFocusedField={setFocusedField} />
      </>
    );
  } else return null;
}
