import FormControl from "react-bootstrap/FormControl";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export default function MinBatchSize(props: Props) {
  return (
    <TaskFormGroup controlId="min_batch_size">
      <ShortHelpAndLabel
        fieldKey="min_batch_size"
        setFocusedField={props.setFocusedField}
      />
      <FormControl
        type="number"
        name="min_batch_size"
        min="300"
        value={props.values.min_batch_size}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        isInvalid={!!props.errors.min_batch_size}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.min_batch_size}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
