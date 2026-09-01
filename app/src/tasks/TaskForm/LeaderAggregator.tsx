import { Await, useLoaderData } from "react-router-dom";
import { FormControl, FormSelect } from "react-bootstrap";
import { Suspense } from "react";
import { Aggregator } from "../../ApiClient.js";
import { Props, TaskFormGroup } from "./index.js";
import { ShortHelpAndLabel } from "./HelpText.js";

export default function LeaderAggregator(props: Props) {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };
  const { helper_aggregator_id } = props.values;

  return (
    <TaskFormGroup controlId="leader_aggregator_id">
      <ShortHelpAndLabel
        fieldKey="leader_aggregator_id"
        setFocusedField={props.setFocusedField}
      />

      <FormSelect
        value={props.values.leader_aggregator_id}
        isInvalid={!!props.errors.leader_aggregator_id}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        name="leader_aggregator_id"
      >
        <option />

        <Suspense>
          <Await resolve={aggregators}>
            {(aggregators: Aggregator[]) =>
              aggregators
                .filter(({ role }) => role === "Leader" || role === "Either")
                .map((aggregator) => (
                  <option
                    key={aggregator.id}
                    value={aggregator.id}
                    disabled={aggregator.id === helper_aggregator_id}
                  >
                    {aggregator.name}
                  </option>
                ))
            }
          </Await>
        </Suspense>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.leader_aggregator_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
