import { useFetcher, useNavigation } from "react-router-dom";
import React, { useEffect, useState } from "react";
import { ArrowRepeat } from "react-bootstrap-icons";
import {
  Button,
  FormControl,
  FormGroup,
  FormLabel,
  Modal,
} from "react-bootstrap";
import { WithAggregator } from "./AggregatorDetail";
import { UpdateAggregator, formikErrors } from "../ApiClient";
import { FormikErrors } from "formik";

export default function RotateBearerTokenButton() {
  const navigation = useNavigation();
  const [show, setShow] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);
  const fetcher = useFetcher();
  const [errors, setErrors] = React.useState(
    null as null | FormikErrors<UpdateAggregator>,
  );

  useEffect(() => {
    if (fetcher.data) {
      if ("error" in fetcher.data) {
        setErrors(formikErrors(fetcher.data.error));
      } else {
        close();
        setErrors(null);
      }
    }
  }, [fetcher.data, close]);

  return (
    <>
      <Button
        variant="outline-secondary"
        className="ml-auto"
        size="sm"
        onClick={open}
      >
        <ArrowRepeat /> Rotate Token
      </Button>
      <Modal show={show} onHide={close}>
        <fetcher.Form method="PATCH">
          <Modal.Header closeButton>
            <Modal.Title>
              Rotate Bearer Token for{" "}
              <WithAggregator>{({ name }) => `"${name}"`}</WithAggregator>
            </Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FormGroup controlId="bearer_token">
              <FormLabel>New Bearer Token</FormLabel>
              <FormControl
                name="bearer_token"
                type="text"
                isInvalid={!!errors?.bearer_token}
              />
              <FormControl.Feedback type="invalid">
                {errors?.bearer_token}
              </FormControl.Feedback>
            </FormGroup>
          </Modal.Body>
          <Modal.Footer>
            <Button variant="secondary" onClick={close}>
              Close
            </Button>
            <Button
              variant="primary"
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              <ArrowRepeat /> Rotate
            </Button>
          </Modal.Footer>
        </fetcher.Form>
      </Modal>
    </>
  );
}
