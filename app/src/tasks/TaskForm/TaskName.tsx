import FormControl from "react-bootstrap/FormControl";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

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
