import Col from "react-bootstrap/Col";
import FormControl from "react-bootstrap/FormControl";
import FormSelect from "react-bootstrap/FormSelect";
import React, { ChangeEvent } from "react";
import Row from "react-bootstrap/Row";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export const seconds = {
  minute: 60,
  hour: 60 * 60,
  day: 60 * 60 * 24,
  week: 60 * 60 * 24 * 7,
};
export type Unit = keyof typeof seconds;

export default function TimePrecisionSeconds(props: Props) {
  const { setFieldValue } = props;
  const [count, setCount] = React.useState<number | undefined>(undefined);
  const [unit, setUnit] = React.useState<Unit>("minute");

  React.useEffect(() => {
    if (typeof count === "number" && unit in seconds) {
      setFieldValue("time_precision_seconds", seconds[unit] * count);
    } else {
      setFieldValue("time_precision_seconds", undefined);
    }
  }, [count, unit, setFieldValue]);

  const changeUnit = React.useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setUnit(event.target.value as Unit);
    },
    [setUnit],
  );

  const changeCount = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setCount(event.target.valueAsNumber);
    },
    [setCount],
  );

  return (
    <TaskFormGroup>
      <ShortHelpAndLabel
        fieldKey="time_precision_seconds"
        setFocusedField={props.setFocusedField}
      />
      <Row>
        <Col xs="2">
          <FormControl
            type="number"
            value={count || ""}
            id="time-precision-number"
            onChange={changeCount}
            isInvalid={!!props.errors.time_precision_seconds}
          />
        </Col>
        <Col>
          <FormSelect
            value={unit}
            onChange={changeUnit}
            isInvalid={!!props.errors.time_precision_seconds}
            id="time-precision-unit"
          >
            {Object.keys(seconds).map((unit) => (
              <option key={unit} value={unit}>
                {unit}
                {count === 1 ? "" : "s"}
              </option>
            ))}
          </FormSelect>
          <FormControl.Feedback type="invalid">
            {props.errors.time_precision_seconds}
          </FormControl.Feedback>
        </Col>
      </Row>
    </TaskFormGroup>
  );
}
