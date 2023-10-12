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

export default function CollectorCredentialForm() {
  const fetcher = useFetcher();
  const [name, setName] = React.useState("");
  const [collectorCredential, setCollectorCredential] = React.useState("");
  const reader = React.useMemo(() => {
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      if (typeof reader.result === "string") {
        setCollectorCredential(reader.result.split(",")[1]);
      }
    });
    return reader;
  }, [setCollectorCredential]);
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
      setCollectorCredential("");
      if (ref.current) {
        ref.current.value = "";
      }
    }
  }, [fetcher, setName, setCollectorCredential, ref]);

  const errors = formikErrors<{ hpke_config?: string; name?: string }>(
    fetcher.data && "error" in fetcher.data
      ? fetcher.data.error
      : { name: undefined, hpke_config: undefined },
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
              isInvalid={!!errors.hpke_config}
              ref={ref}
            />
            {errors.hpke_config ? (
              <FormControl.Feedback type="invalid">
                {errors.hpke_config}
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
          <input type="hidden" name="hpke_config" value={collectorCredential} />
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
