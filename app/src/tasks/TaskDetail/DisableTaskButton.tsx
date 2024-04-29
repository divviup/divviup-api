import {
  Form,
  useFetcher,
  useLoaderData,
  useNavigation,
  useParams,
} from "react-router-dom";
import React, { FormEvent, useCallback, useEffect, useState } from "react";
import { Play, SignStop } from "react-bootstrap-icons";
import { Button, Modal } from "react-bootstrap";
import { WithTask } from ".";
import { Task } from "../../ApiClient";

export default function DisableTaskButton() {
  const { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  const navigation = useNavigation();
  const [show, setShow] = useState(false);
  const [isExpired, setIsExpired] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);
  const fetcher = useFetcher();

  const { taskId, accountId } = useParams() as {
    taskId: string;
    accountId: string;
  };

  const submit = useCallback(
    (e: FormEvent<HTMLFormElement>) => {
      e.preventDefault();
      fetcher.submit(
        { expiration: isExpired ? null : new Date().toISOString() },
        {
          action: `/accounts/${accountId}/tasks/${taskId}`,
          method: "PATCH",
          encType: "application/json",
        },
      );
    },
    [fetcher, taskId, accountId, isExpired],
  );

  const checkExpiration = useCallback(async () => {
    const expiration = (await task).expiration;
    if (expiration === null) {
      setIsExpired(false);
    } else {
      const parsedExpiration = Date.parse(expiration as string);
      if (parsedExpiration <= Date.now()) {
        setIsExpired(true);
      } else {
        setIsExpired(false);
      }
    }
  }, [task]);

  useEffect(() => {
    checkExpiration();
    if (fetcher.data) close();
  }, [fetcher, close, checkExpiration]);

  let verb, variant, body, sign;
  if (isExpired) {
    verb = "Enable";
    sign = <Play />;
    variant = "success";
    body = (
      <>
        Aggregators will accept reports for this task. It may take a few minutes
        for this to take effect.
      </>
    );
  } else {
    verb = "Disable";
    sign = <SignStop />;
    variant = "warning";
    body = (
      <>
        Aggregators will stop accepting reports for this task. It may take a few
        minutes for this to take effect.
      </>
    );
  }

  return (
    <>
      <Button variant={variant} size="lg" onClick={open}>
        {sign} {verb}
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>
            {verb} <WithTask>{({ name }) => `"${name}"`}</WithTask>?
          </Modal.Title>
        </Modal.Header>
        <Modal.Body>{body}</Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <Form onSubmit={submit}>
            <Button
              variant={variant}
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              {sign} {verb}
            </Button>
          </Form>
        </Modal.Footer>
      </Modal>
    </>
  );
}
