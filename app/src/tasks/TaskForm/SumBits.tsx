import FormSelect from "react-bootstrap/FormSelect";
import React, { ChangeEvent } from "react";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export function SumBits(props: Props) {
  const { setFieldValue } = props;
  const handleChange = React.useCallback(
    (event: ChangeEvent<HTMLSelectElement>) =>
      setFieldValue("vdaf.bits", parseInt(event.target.value, 10)),
    [setFieldValue],
  );
  if (props.values.vdaf?.type !== "sum") return <></>;

  return (
    <TaskFormGroup controlId="vdaf.bits">
      <ShortHelpAndLabel
        fieldKey="vdaf.bits"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        value={props.values.vdaf?.bits}
        name="vdaf.bits"
        onChange={handleChange}
        onBlur={props.handleBlur}
      >
        {[8, 16, 32, 64].map((i) => (
          <option value={i} key={i}>
            Unsigned {i}-bit integer (0 to{" "}
            {(Math.pow(2, i) - 1).toLocaleString()})
          </option>
        ))}
      </FormSelect>
    </TaskFormGroup>
  );
}
