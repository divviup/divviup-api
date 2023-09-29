import { useLoaderData } from "react-router-dom";
import FormControl from "react-bootstrap/FormControl";
import FormSelect from "react-bootstrap/FormSelect";
import { Aggregator } from "../../ApiClient";
import { usePromise } from "../../util";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export default function VdafType(props: Props) {
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

  const vdafs = new Set(
    leader && helper
      ? leader.vdafs
          .filter((vdaf) => helper.vdafs.includes(vdaf))
          .map((vdaf) => vdaf.replace(/^Prio3/, "").toLowerCase())
      : ["sum", "histogram", "count"],
  );

  return (
    <TaskFormGroup controlId="vdaf.type">
      <ShortHelpAndLabel
        fieldKey="vdaf.type"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        value={props.values.vdaf?.type}
        name="vdaf.type"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        isInvalid={typeof props.errors.vdaf === "string"}
      >
        <option></option>
        {["sum", "histogram", "count"].map((vdaf) => (
          <option key={vdaf} value={vdaf} disabled={!vdafs.has(vdaf)}>
            {vdaf}
          </option>
        ))}
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {typeof props.errors.vdaf === "string" ? props.errors.vdaf : null}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
