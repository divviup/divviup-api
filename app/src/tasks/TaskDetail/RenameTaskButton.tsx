import { useFetcher, useParams } from "react-router-dom";
import { FormEvent, useCallback, useEffect, useState } from "react";
import { Pencil, PencilSquare } from "react-bootstrap-icons";
import {
  Button,
  FormControl,
  FormGroup,
  FormLabel,
  Modal,
} from "react-bootstrap";
import { UpdateTask, formikErrors } from "../../ApiClient";
import { FormikErrors } from "formik";
import { WithTask } from ".";

export default function RenameTaskButton() {
  const [show, setShow] = useState(false);
  const close = useCallback(() => setShow(false), []);
  const open = useCallback(() => setShow(true), []);
  const fetcher = useFetcher();
  const [errors, setErrors] = useState(null as null | FormikErrors<UpdateTask>);

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
      <Button size="lg" onClick={open}>
        <PencilSquare /> Rename
      </Button>
      <Modal show={show} onHide={close}>
        <fetcher.Form method="PATCH">
          <Modal.Header closeButton>
            <Modal.Title>
              Rename <WithTask>{({ name }) => `"${name}"`}</WithTask>
            </Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FormGroup controlId="name">
              <FormLabel>Name</FormLabel>
              <WithTask>
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
              </WithTask>
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
              <Pencil /> Rename
            </Button>
          </Modal.Footer>
        </fetcher.Form>
      </Modal>
    </>
  );
}
