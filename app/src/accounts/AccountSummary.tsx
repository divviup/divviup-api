import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import { useFetcher, useParams, useRouteLoaderData } from "react-router-dom";
import {
  ChangeEvent,
  FormEvent,
  Suspense,
  useCallback,
  useEffect,
  useState,
} from "react";
import { LinkContainer } from "react-router-bootstrap";
import {
  Building,
  CloudUpload,
  FileEarmarkCode,
  KeyFill,
  PencilFill,
  People,
  ShieldLock,
} from "react-bootstrap-icons";
import { Button, Card, FormControl, InputGroup } from "react-bootstrap";
import { WithAccount } from "../util";
import Placeholder from "react-bootstrap/Placeholder";
import NextSteps from "./NextSteps";
import { Account } from "../ApiClient";

function AccountName() {
  const [isEditingName, setIsEditingName] = useState(false);
  const edit = useCallback(() => setIsEditingName(true), [setIsEditingName]);
  const { accountId } = useParams() as { accountId: string };
  const [name, setName] = useState("");
  const [originalName, setOriginalName] = useState(name);
  const { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };
  useEffect(() => {
    account.then(({ name }) => {
      setOriginalName(name);
      setName(name);
      setIsEditingName(false);
    });
  }, [account]);
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
            action: `/accounts/${accountId}`, // this is necessary because the current route is ?index
            method: "PATCH",
            encType: "application/json",
          },
        );
      }
    },
    [fetcher, originalName, name, accountId],
  );

  if (isEditingName) {
    return (
      <form onSubmit={submit}>
        <Row>
          <Col xs="11">
            <InputGroup hasValidation>
              <InputGroup.Text id="inputGroupPrepend">
                <Building />
              </InputGroup.Text>
              <FormControl
                type="text"
                name="name"
                data-1p-ignore
                value={name}
                required
                onChange={change}
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
      <Row>
        <Col xs="11">
          <h1>
            <Building />
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>
          </h1>
        </Col>
        <Col>
          <Button variant="secondary" onClick={edit}>
            <PencilFill />
          </Button>
        </Col>
      </Row>
    );
  }
}

export default function AccountSummary() {
  return (
    <>
      <Row>
        <Col>
          <Breadcrumb>
            <LinkContainer to="/accounts">
              <Breadcrumb.Item>Accounts</Breadcrumb.Item>
            </LinkContainer>
            <Breadcrumb.Item active>
              <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
                <WithAccount>{(account) => account.name}</WithAccount>
              </Suspense>
            </Breadcrumb.Item>
          </Breadcrumb>
        </Col>
      </Row>
      <Row>
        <Col>
          <AccountName />
        </Col>
      </Row>
      <NextSteps />
      <Row>
        <Col sm="4">
          <Card className="my-3 shadow">
            <Card.Body>
              <Card.Title>Quick Links</Card.Title>
            </Card.Body>

            <ListGroup variant="flush">
              <LinkContainer to="memberships">
                <ListGroup.Item action>
                  <People /> Members
                </ListGroup.Item>
              </LinkContainer>

              <LinkContainer to="tasks">
                <ListGroup.Item action>
                  <FileEarmarkCode /> Tasks
                </ListGroup.Item>
              </LinkContainer>

              <LinkContainer to="aggregators">
                <ListGroup.Item action>
                  <CloudUpload /> Aggregators
                </ListGroup.Item>
              </LinkContainer>

              <LinkContainer to="api_tokens">
                <ListGroup.Item action>
                  <ShieldLock /> API Tokens
                </ListGroup.Item>
              </LinkContainer>

              <LinkContainer to="hpke_configs">
                <ListGroup.Item action>
                  <KeyFill /> HPKE Configs
                </ListGroup.Item>
              </LinkContainer>
            </ListGroup>
          </Card>
        </Col>

        <Col sm="4">
          <Card className="my-3 shadow">
            <Card.Body>
              <Card.Title>Billing Metrics</Card.Title>
            </Card.Body>
            <ListGroup variant="flush">
              {[...new Array(4)].map((_, n) => (
                <ListGroup.Item key={n} className="placeholder-wave">
                  <Placeholder className="col-7" />
                </ListGroup.Item>
              ))}
            </ListGroup>
          </Card>
        </Col>

        <Col sm="4">
          <Card className="my-3 shadow">
            <Card.Body>
              <Card.Title>System Messages</Card.Title>
            </Card.Body>
            <ListGroup variant="flush">
              {[...new Array(6)].map((_, n) => (
                <ListGroup.Item key={n} className="placeholder-wave">
                  <Placeholder className="col-7" />
                </ListGroup.Item>
              ))}
            </ListGroup>
          </Card>
        </Col>
      </Row>
    </>
  );
}
