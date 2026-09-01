import { FormControl } from "react-bootstrap";
import { Props, TaskFormGroup } from "./index.js";
import { ShortHelpAndLabel } from "./HelpText.js";

export default function TaskName(props: Props) {
  return (
    <TaskFormGroup controlId="name">
      <ShortHelpAndLabel
        fieldKey="name"
        setFocusedField={props.setFocusedField}
      />
      <FormControl
        type="text"
        name="name"
        autoComplete="off"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.name}
        isInvalid={!!props.errors.name}
        data-1p-ignore
      />
      <FormControl.Feedback type="invalid">
        {props.errors.name}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
