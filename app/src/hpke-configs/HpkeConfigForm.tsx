import React, {
  ChangeEvent,
  ChangeEventHandler,
  useEffect,
  useRef,
} from "react";
import {
  Button,
  Col,
  FormControl,
  FormGroup,
  FormLabel,
  Row,
} from "react-bootstrap";
import { KeyFill } from "react-bootstrap-icons";
import { useFetcher } from "react-router-dom";
import { formikErrors } from "../ApiClient";

export default function HpkeConfigForm() {
  const fetcher = useFetcher();
  const [name, setName] = React.useState("");
  const [hpke, setHpke] = React.useState("");
  const reader = React.useMemo(() => {
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      if (typeof reader.result === "string") {
        setHpke(reader.result.split(",")[1]);
      }
    });
    return reader;
  }, [setHpke]);
  const onChange: ChangeEventHandler<HTMLInputElement> = React.useCallback(
    (event) => {
      const files = event.target.files;
      if (files && files[0]) {
        if (!name) setName(files[0].name);
        reader.readAsDataURL(files[0]);
      }
    },
    [reader, setName, name],
  );

  const ref = useRef<HTMLInputElement | null>(null);
  useEffect(() => {
    if (typeof fetcher.data === "object" && !("error" in fetcher.data)) {
      setName("");
      setHpke("");
      if (ref.current) {
        ref.current.value = "";
      }
    }
  }, [fetcher, setName, setHpke, ref]);

  const errors = formikErrors<{ contents?: string; name?: string }>(
    fetcher.data && "error" in fetcher.data
      ? fetcher.data.error
      : { name: undefined, contents: undefined },
  );

  return (
    <fetcher.Form method="post">
      <Row>
        <Col sm="5">
          <FormGroup>
            <FormLabel>DAP-encoded HPKE config file</FormLabel>
            <FormControl
              type="file"
              onChange={onChange}
              isInvalid={!!errors.contents}
              ref={ref}
            />
            {errors.contents ? (
              <FormControl.Feedback type="invalid">
                {errors.contents}
              </FormControl.Feedback>
            ) : null}
          </FormGroup>
        </Col>
        <Col sm="5">
          <FormGroup controlId="name">
            <FormLabel>Config Name</FormLabel>
            <FormControl
              type="text"
              name="name"
              data-1p-ignore
              value={name}
              onChange={React.useCallback(
                (event: ChangeEvent<HTMLInputElement>) =>
                  setName(event.target.value),
                [setName],
              )}
              isInvalid={!!errors.name}
            />
            {errors.name ? (
              <FormControl.Feedback type="invalid">
                {errors.name}
              </FormControl.Feedback>
            ) : null}
          </FormGroup>
          <input type="hidden" name="contents" value={hpke} />
        </Col>
        <Col sm="2">
          <FormGroup controlId="submit" className="my-3">
            <Button
              variant="primary"
              type="submit"
              className="my-3"
              disabled={fetcher.state === "submitting"}
            >
              <KeyFill /> Upload
            </Button>
          </FormGroup>
        </Col>
      </Row>
    </fetcher.Form>
  );
}
