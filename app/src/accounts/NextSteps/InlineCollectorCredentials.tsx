import { Button, Col, Row } from "react-bootstrap";
import { useFetcher, useParams, useRevalidator } from "react-router-dom";
import { ApiToken } from "../../ApiClient";
import React from "react";
import useInterval from "use-interval";
import { ApiClientContext } from "../../ApiClientContext";
import {
  ClipboardCheckFill,
  Icon1CircleFill,
  Icon2CircleFill,
  Icon3CircleFill,
  Clipboard,
} from "react-bootstrap-icons";
import { Copy, OutLink } from "../../util";

function SaveApiToken({ onToken }: { onToken: (token: string) => void }) {
  const fetcher = useFetcher<ApiToken & { token: string }>();

  React.useEffect(() => {
    if (fetcher.state === "idle" && fetcher.data) onToken(fetcher.data.token);
  }, [fetcher.data, fetcher.state]);

  if (fetcher.state === "idle" && fetcher.data) {
    const { token } = fetcher.data;
    return (
      <Copy clipboardContents={fetcher.data.token}>
        {(copy, copied) => (
          <span onClick={copy} style={{ cursor: "pointer" }}>
            <code className="user-select-all">{token}</code>{" "}
            <Button size="sm" variant="outline-secondary" className="ml-auto">
              {copied ? <ClipboardCheckFill /> : <Clipboard />}
            </Button>
          </span>
        )}
      </Copy>
    );
  } else {
    return (
      <fetcher.Form
        method="POST"
        action="./api_tokens?index"
        className="d-inline-block"
      >
        <Button
          variant="primary"
          size="sm"
          type="submit"
          disabled={fetcher.state === "submitting"}
        >
          Generate Now
        </Button>
      </fetcher.Form>
    );
  }
}

export default function InlineCollectorCredentials() {
  const apiClient = React.useContext(ApiClientContext);
  const { accountId } = useParams() as { accountId: string };
  const [anyCollectorCredentials, setAnyCollectorCredentials] =
    React.useState(false);
  const [inFlight, setInFlight] = React.useState(false);
  useInterval(
    React.useCallback(() => {
      if (inFlight || anyCollectorCredentials) return;
      setInFlight(true);
      apiClient
        .accountCollectorCredentials(accountId)
        .then((collectorCredentials) => {
          setAnyCollectorCredentials(collectorCredentials.length > 0);
          setInFlight(false);
        });
    }, [apiClient, setInFlight, setAnyCollectorCredentials]),
    1000,
  );

  const [token, setToken] = React.useState<string>("«TOKEN»");
  const { revalidate, state } = useRevalidator();

  if (anyCollectorCredentials) {
    return (
      <>
        <h1>Success</h1>
        <Button onClick={revalidate} disabled={state === "loading"}>
          Next
        </Button>
      </>
    );
  } else {
    const command = `divviup -t ${token} collector-credential generate`;
    return (
      <>
        <ol className="list-unstyled">
          <li className="my-5">
            <Row>
              <Col sm="1" className="text-center">
                <Icon1CircleFill color="#3344B9" size={35} />
              </Col>
              <Col>
                Generate API Token{" "}
                <div>
                  <SaveApiToken onToken={setToken} />
                </div>
                <small className="d-block">
                  <em>
                    Once generated, store this somewhere safe, like a password
                    manager or secrets vault. Divvi Up does not store a copy.
                  </em>
                </small>
              </Col>
            </Row>
          </li>
          <li className="my-5">
            <Row>
              <Col sm="1" className="text-center">
                <Icon2CircleFill color="#3344B9" size={35} />
              </Col>
              <Col>
                Install the Divvi Up CLI{" "}
                <OutLink href="https://docs.divviup.org">
                  View Documentation
                </OutLink>
              </Col>
            </Row>
          </li>
          <li className="my-5">
            <Row>
              <Col sm="1" className="text-center">
                <Icon3CircleFill color="#3344B9" size={35} />
              </Col>
              <Col>
                Use token to generate collector credentials:
                <div>
                  <Copy clipboardContents={command}>
                    {(copy, copied) => (
                      <span onClick={copy} style={{ cursor: "pointer" }}>
                        <code className="user-select-all">{command}</code>{" "}
                        <Button
                          size="sm"
                          variant="outline-secondary"
                          className="ml-auto"
                        >
                          {copied ? <ClipboardCheckFill /> : <Clipboard />}
                        </Button>
                      </span>
                    )}
                  </Copy>
                </div>
              </Col>
            </Row>
          </li>
        </ol>
      </>
    );
  }
}
