import { Await, useParams, useLoaderData, useFetcher } from "react-router-dom";
import Col from "react-bootstrap/Col";
import React, {
  ChangeEvent,
  FormEvent,
  useCallback,
  useEffect,
  useState,
} from "react";
import Row from "react-bootstrap/Row";
import { Task } from "../../ApiClient";
import { Pencil, PencilFill } from "react-bootstrap-icons";
import Button from "react-bootstrap/Button";
import FormControl from "react-bootstrap/FormControl";
import InputGroup from "react-bootstrap/InputGroup";
import { VdafIcon } from "../VdafIcon";
import { Placeholder } from "react-bootstrap";

export default function TaskTitle() {
  const [isEditingName, setIsEditingName] = useState(false);
  const edit = useCallback(() => setIsEditingName(true), [setIsEditingName]);
  const { taskId, accountId } = useParams() as {
    taskId: string;
    accountId: string;
  };
  const [name, setName] = useState("");
  const [originalName, setOriginalName] = useState(name);
  const { task } = useLoaderData() as {
    task: Promise<Task>;
  };
  useEffect(() => {
    task.then(({ name }) => {
      setOriginalName(name);
      setName(name);
      setIsEditingName(false);
    });
  }, [task]);
  const change = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => setName(e.target.value),
    [setName],
  );
  const fetcher = useFetcher();
  const submit = useCallback(
    (e: FormEvent<HTMLFormElement>) => {
      e.preventDefault();
      if (name === originalName) {
        setIsEditingName(false);
      } else {
        fetcher.submit(
          { name },
          {
            action: `/accounts/${accountId}/tasks/${taskId}`,
            method: "PATCH",
            encType: "application/json",
          },
        );
      }
    },
    [fetcher, name, originalName, accountId, taskId],
  );

  if (isEditingName) {
    return (
      <form onSubmit={submit}>
        <Row>
          <Col xs="10">
            <InputGroup hasValidation>
              <React.Suspense>
                <Await resolve={task}>
                  {(task: Task) => (
                    <InputGroup.Text id="inputGroupPrepend">
                      <VdafIcon fill task={task} />
                    </InputGroup.Text>
                  )}
                </Await>
              </React.Suspense>
              <FormControl
                type="text"
                name="name"
                data-1p-ignore
                value={name}
                onChange={change}
                required
              />
            </InputGroup>
          </Col>
          <Col>
            <Button variant="primary" type="submit">
              <PencilFill />
            </Button>
          </Col>
        </Row>
      </form>
    );
  } else {
    return (
      <React.Suspense>
        <Await resolve={task}>
          {(task: Task) => {
            return (
              <Row>
                <Col xs="10">
                  <h1>
                    <React.Suspense fallback={<Placeholder />}>
                      <Await resolve={task}>
                        {(task) => (
                          <>
                            <VdafIcon fill task={task} /> {task.name}
                          </>
                        )}
                      </Await>
                    </React.Suspense>
                  </h1>
                </Col>
                <Col>
                  <Button onClick={edit}>
                    <Pencil className="text-end" /> Edit Title
                  </Button>
                </Col>
              </Row>
            );
          }}
        </Await>
      </React.Suspense>
    );
  }
}
