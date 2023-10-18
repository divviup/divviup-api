import { useFetcher } from "react-router-dom";
import React from "react";
import { Pencil, PencilSquare } from "react-bootstrap-icons";
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

export default function RenameAggregatorButton() {
  const [show, setShow] = React.useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);
  const fetcher = useFetcher();
  const [errors, setErrors] = React.useState(
    null as null | FormikErrors<UpdateAggregator>,
  );

  React.useEffect(() => {
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
        <PencilSquare /> Rename
      </Button>
      <Modal show={show} onHide={close}>
        <fetcher.Form method="PATCH">
          <Modal.Header closeButton>
            <Modal.Title>
              Rename{" "}
              <WithAggregator>{({ name }) => `"${name}"`}</WithAggregator>
            </Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FormGroup controlId="name">
              <FormLabel>Name</FormLabel>
              <WithAggregator>
                {({ name }) => (
                  <>
                    <FormControl
                      name="name"
                      type="text"
                      data-1p-ignore
                      defaultValue={name}
                      isInvalid={!!errors?.name}
                    />
                    <FormControl.Feedback type="invalid">
                      {errors?.name}
                    </FormControl.Feedback>
                  </>
                )}
              </WithAggregator>
            </FormGroup>
          </Modal.Body>
          <Modal.Footer>
            <Button variant="secondary" onClick={close}>
              Close
            </Button>
            <Button
              variant="primary"
              type="submit"
              disabled={fetcher.state === "submitting"}
            >
              <Pencil /> Edit
            </Button>
          </Modal.Footer>
        </fetcher.Form>
      </Modal>
    </>
  );
}
