import FormControl from "react-bootstrap/FormControl";
import FormLabel from "react-bootstrap/FormLabel";
import React, { ChangeEvent } from "react";
import { Props, TaskFormGroup } from ".";

export function MaxBatchSize(props: Props) {
  const { values, setFieldValue, errors, handleBlur } = props;
  const { max_batch_size, min_batch_size } = values;

  const handleChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (event.target.value) {
        setFieldValue("max_batch_size", event.target.valueAsNumber);
      }
    },
    [setFieldValue],
  );

  if (typeof max_batch_size !== "number") return null;
  return (
    <TaskFormGroup>
      <FormLabel>Maximum Batch Size</FormLabel>
      <FormControl
        type="number"
        name="max_batch_size"
        value={max_batch_size}
        onChange={handleChange}
        step={1}
        onBlur={handleBlur}
        min={min_batch_size}
        isInvalid={!!errors.max_batch_size}
      />
      <FormControl.Feedback type="invalid">
        {errors.max_batch_size}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
