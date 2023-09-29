import FormControl from "react-bootstrap/FormControl";
import React, { ChangeEvent } from "react";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export function HistogramBucketSelection(props: Props) {
  const { setFieldValue } = props;
  const [input, setInput] = React.useState(
    props.values.vdaf?.type === "histogram"
      ? (props.values.vdaf?.buckets || []).join(", ")
      : "",
  );

  const cb = React.useCallback(
    (change: ChangeEvent<HTMLInputElement>) => {
      if (/^([0-9]+, *)*[0-9]*$/.test(change.target.value)) {
        if (input.length) {
          setFieldValue(
            "vdaf.buckets",
            input
              .split(/, */)
              .map((n) => parseInt(n, 10))
              .filter((n) => !isNaN(n)),
          );
        }
        setInput(change.target.value);
      } else {
        change.stopPropagation();
        change.preventDefault();
      }
    },
    [input, setInput, setFieldValue],
  );

  const blur = React.useCallback(() => {
    let value = input
      .split(/, */)
      .map((n) => parseInt(n, 10))
      .sort((a, b) => a - b);
    value = [...new Set(value)];
    setInput(value.join(", "));
    setFieldValue("vdaf.buckets", value);
  }, [input, setInput, setFieldValue]);

  if (props.values.vdaf?.type !== "histogram") return <></>;
  return (
    <TaskFormGroup controlId="vdaf.buckets">
      <ShortHelpAndLabel
        fieldKey="vdaf.buckets"
        setFocusedField={props.setFocusedField}
      />

      <FormControl
        value={input}
        name="vdaf.buckets"
        onChange={cb}
        onBlur={blur}
        isInvalid={
          typeof props.errors.vdaf === "object" &&
          "buckets" in props.errors.vdaf &&
          !!props.errors.vdaf.buckets
        }
      />
      <FormControl.Feedback type="invalid">
        {typeof props.errors.vdaf === "object" &&
          "buckets" in props.errors.vdaf &&
          props.errors.vdaf.buckets}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
