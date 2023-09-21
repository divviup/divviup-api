import { useLoaderData } from "react-router-dom";
import React, { ChangeEvent } from "react";
import { Aggregator } from "../../ApiClient";
import FormCheck from "react-bootstrap/FormCheck";
import { usePromise } from "../../util";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";
import { MaxBatchSize } from "./MaxBatchSize";

export default function QueryType(props: Props) {
  const {
    setFieldValue,
    values: { max_batch_size, min_batch_size },
  } = props;
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };

  const { leader_aggregator_id, helper_aggregator_id } = props.values;
  const aggregatorsResolved = usePromise(aggregators, []);
  const leader = leader_aggregator_id
    ? aggregatorsResolved.find(({ id }) => id === leader_aggregator_id) || null
    : null;
  const helper = helper_aggregator_id
    ? aggregatorsResolved.find(({ id }) => id === helper_aggregator_id) || null
    : null;
  const queryTypes =
    leader && helper
      ? leader.query_types.filter((qt) => helper.query_types.includes(qt))
      : ["TimeInterval", "FixedSize"];

  const timeInterval =
    queryTypes.includes("TimeInterval") && typeof max_batch_size !== "number";

  const checkboxChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setFieldValue(
        "max_batch_size",
        event.target.value === "fixed" && typeof min_batch_size === "number"
          ? min_batch_size * 2
          : /*jbr: I have no idea what a good
             * default is, but it needs to be
             * greater than min*/

            null,
      );
    },
    [setFieldValue, min_batch_size],
  );

  return (
    <TaskFormGroup controlId="querytype">
      <ShortHelpAndLabel
        fieldKey="max_batch_size"
        setFocusedField={props.setFocusedField}
      />
      <FormCheck
        type="radio"
        name="query-type"
        id="query=type-time"
        checked={timeInterval}
        onChange={checkboxChange}
        label="Time Interval"
        disabled={!queryTypes.includes("TimeInterval")}
        value="time"
      />
      <FormCheck
        type="radio"
        name="query-type"
        id="query-type-fixed"
        checked={!timeInterval}
        onChange={checkboxChange}
        label="Fixed Size"
        disabled={!queryTypes.includes("FixedSize")}
        value="fixed"
      />
      <MaxBatchSize {...props} />
    </TaskFormGroup>
  );
}
