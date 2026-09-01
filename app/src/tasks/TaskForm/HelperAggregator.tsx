import { Await, useLoaderData } from "react-router-dom";
import { FormControl, FormSelect } from "react-bootstrap";
import { Suspense } from "react";
import { Aggregator } from "../../ApiClient.js";
import { Props, TaskFormGroup } from "./index.js";
import { ShortHelpAndLabel } from "./HelpText.js";

export default function HelperAggregator(props: Props) {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };
  const { leader_aggregator_id } = props.values;
  return (
    <TaskFormGroup>
      <ShortHelpAndLabel
        fieldKey="helper_aggregator_id"
        setFocusedField={props.setFocusedField}
      />

      <FormSelect
        value={props.values.helper_aggregator_id}
        isInvalid={!!props.errors.helper_aggregator_id}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        name="helper_aggregator_id"
      >
        <option />
        <Suspense>
          <Await resolve={aggregators}>
            {(aggregators: Aggregator[]) =>
              aggregators
                .filter(({ role }) => role === "Helper" || role === "Either")
                .map((aggregator) => (
                  <option
                    key={aggregator.id}
                    value={aggregator.id}
                    disabled={aggregator.id === leader_aggregator_id}
                  >
                    {aggregator.name}
                  </option>
                ))
            }
          </Await>
        </Suspense>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.helper_aggregator_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
