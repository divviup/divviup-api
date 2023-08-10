import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import { Form, useActionData } from "react-router-dom";
import { Suspense, useCallback, useEffect, useState } from "react";
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
import { Button, FormControl, InputGroup } from "react-bootstrap";
import { WithAccount } from "../util";
import Placeholder from "react-bootstrap/Placeholder";

function AccountName() {
  let [isEditingName, setIsEditingName] = useState(false);
  let edit = useCallback(() => setIsEditingName(true), [setIsEditingName]);
  let actionData = useActionData();
  useEffect(() => {
    if (actionData) setIsEditingName(false);
  }, [actionData]);

  if (isEditingName) {
    return (
      <>
        <Row>
          <Col xs="11">
            <Form method="patch">
              <InputGroup hasValidation>
                <InputGroup.Text id="inputGroupPrepend">
                  <Building />
                </InputGroup.Text>
                <Suspense
                  fallback={<FormControl type="text" name="name" required />}
                >
                  <WithAccount>
                    {(account) => (
                      <FormControl
                        type="text"
                        name="name"
                        defaultValue={account.name}
                        required
                      />
                    )}
                  </WithAccount>
                </Suspense>
              </InputGroup>
            </Form>
          </Col>
          <Col>
            <Button variant="primary" type="submit">
              <PencilFill />
            </Button>
          </Col>
        </Row>
      </>
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
          <ListGroup>
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
        </Col>
      </Row>
    </>
  );
}
