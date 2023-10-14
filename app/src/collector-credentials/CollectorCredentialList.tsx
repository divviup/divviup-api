import Breadcrumb from "react-bootstrap/Breadcrumb";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Button from "react-bootstrap/Button";
import { AccountBreadcrumbs, WithAccount } from "../util";
import { Check, PencilFill, Key, Trash } from "react-bootstrap-icons";
import { Suspense, useCallback, useEffect, useState } from "react";
import {
  Await,
  useFetcher,
  useLoaderData,
  useNavigation,
} from "react-router-dom";
import { CollectorCredential } from "../ApiClient";
import Table from "react-bootstrap/Table";
import React from "react";
import { DateTime } from "luxon";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import InputGroup from "react-bootstrap/InputGroup";
import Modal from "react-bootstrap/Modal";
import Placeholder from "react-bootstrap/Placeholder";
import CollectorCredentialForm from "./CollectorCredentialForm";

export default function CollectorCredentials() {
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <Key />{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>{" "}
            HPKE Configs
          </h1>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <CollectorCredentialForm />
        </Col>
      </Row>
      <Row>
        <Col>
          <CollectorCredentialList />
        </Col>
      </Row>
    </>
  );
}

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <Breadcrumb.Item active>HPKE Configs</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

function CollectorCredentialList() {
  const { collectorCredentials } = useLoaderData() as {
    collectorCredentials: Promise<CollectorCredential[]>;
  };

  return (
    <Table>
      <thead>
        <tr>
          <td>Name</td>
          <td>KEM</td>
          <td>KDF</td>
          <td>AEAD</td>
          <td>Created</td>
          <td></td>
        </tr>
      </thead>
      <tbody>
        <Suspense>
          <Await resolve={collectorCredentials}>
            {(collectorCredentials: CollectorCredential[]) =>
              collectorCredentials.map((collectorCredential) => (
                <CollectorCredentialRow
                  key={collectorCredential.id}
                  collectorCredential={collectorCredential}
                />
              ))
            }
          </Await>
        </Suspense>
      </tbody>
    </Table>
  );
}

function Name({
  collectorCredential,
}: {
  collectorCredential: CollectorCredential;
}) {
  const [isEditing, setEditing] = useState(false);
  const edit = useCallback(() => setEditing(true), [setEditing]);
  const fetcher = useFetcher();
  useEffect(() => {
    if (fetcher.data) setEditing(false);
  }, [fetcher, setEditing]);
  if (isEditing) {
    return (
      <fetcher.Form action={collectorCredential.id} method="patch">
        <FormGroup>
          <InputGroup>
            <FormControl
              type="text"
              name="name"
              defaultValue={collectorCredential.name || ""}
              data-1p-ignore
              autoFocus
            />
            <Button type="submit">
              <Check />
            </Button>
          </InputGroup>
        </FormGroup>
      </fetcher.Form>
    );
  } else {
    return (
      <span onClick={edit}>
        {collectorCredential.name ||
          `HPKE Config ${collectorCredential.hpke_config.id}`}{" "}
        <Button
          variant="outline-secondary"
          onClick={edit}
          size="sm"
          className="ml-auto"
        >
          <PencilFill />
        </Button>
      </span>
    );
  }
}

function RelativeTime({ time, missing }: { time?: string; missing?: string }) {
  return time ? (
    <relative-time datetime={time} format="relative">
      {DateTime.fromISO(time).toLocal().toLocaleString(DateTime.DATETIME_SHORT)}
    </relative-time>
  ) : (
    <>{missing || "never"}</>
  );
}

function DeleteButton({
  collectorCredential,
}: {
  collectorCredential: CollectorCredential;
}) {
  const navigation = useNavigation();

  const [show, setShow] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);
  const fetcher = useFetcher();

  useEffect(() => {
    if (fetcher.data) close();
  }, [fetcher, close]);

  return (
    <>
      <Button
        variant="outline-danger"
        className="ml-auto"
        size="sm"
        onClick={open}
      >
        <Trash />
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>
            Confirm HPKE Config Deletion {collectorCredential.name}
          </Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This HPKE Config will immediately be inactivated and cannot be used to
          create new tasks. Existing tasks will continue to function.
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <fetcher.Form method="delete" action={collectorCredential.id}>
            <Button
              variant="danger"
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              <Trash /> Delete
            </Button>
          </fetcher.Form>
        </Modal.Footer>
      </Modal>
    </>
  );
}

function CollectorCredentialRow({
  collectorCredential,
}: {
  collectorCredential: CollectorCredential;
}) {
  return (
    <tr>
      <td>
        <Name collectorCredential={collectorCredential} />
      </td>
      <td>{collectorCredential.hpke_config.kem_id}</td>
      <td>{collectorCredential.hpke_config.kdf_id}</td>
      <td>{collectorCredential.hpke_config.aead_id}</td>
      <td>
        <RelativeTime time={collectorCredential.created_at} />
      </td>
      <td>
        <DeleteButton collectorCredential={collectorCredential} />
      </td>
    </tr>
  );
}
