import Breadcrumb from "react-bootstrap/Breadcrumb";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Button from "react-bootstrap/Button";
import { AccountBreadcrumbs, WithAccount } from "../util";
import { CloudUpload } from "react-bootstrap-icons";
import React from "react";
import { LinkContainer } from "react-router-bootstrap";
import { Formik, FormikErrors, FormikHelpers, FormikProps } from "formik";
import Form from "react-bootstrap/Form";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import FormLabel from "react-bootstrap/FormLabel";
import FormSelect from "react-bootstrap/FormSelect";
import ApiClient, { NewAggregator, formikErrors } from "../ApiClient";
import {
  NavigateFunction,
  useActionData,
  useNavigate,
  useNavigation,
  useParams,
} from "react-router-dom";
import { ApiClientContext } from "../ApiClientContext";
const { Suspense } = React;
import Placeholder from "react-bootstrap/Placeholder";

async function submit(
  apiClient: ApiClient,
  accountId: string,
  newAggregator: NewAggregator,
  actions: FormikHelpers<NewAggregator>,
  navigate: NavigateFunction
) {
  try {
    let aggregator = await apiClient.createAggregator(accountId, newAggregator);

    if ("error" in aggregator) {
      actions.setErrors(formikErrors(aggregator.error));
    } else {
      return navigate(`/accounts/${accountId}/aggregators/${aggregator.id}`);
    }
  } catch (e) {
    console.log(e);
  }
}

export default function AggregatorForm() {
  let params = useParams();
  const accountId = params.account_id as string;
  const actionData = useActionData();
  let errors: undefined | FormikErrors<NewAggregator> = undefined;
  if (typeof actionData === "object" && actionData && "error" in actionData) {
    errors = actionData.error as FormikErrors<NewAggregator>;
  }
  const navigate = useNavigate();
  const navigation = useNavigation();
  const apiClient = React.useContext(ApiClientContext);
  const handleSubmit = React.useCallback(
    (values: NewAggregator, actions: FormikHelpers<NewAggregator>) =>
      submit(apiClient, accountId as string, values, actions, navigate),
    [apiClient, accountId, navigate]
  );

  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <CloudUpload />{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>{" "}
            Aggregators
          </h1>
        </Col>
      </Row>
      <Row>
        <Col>
          <Formik
            validateOnChange={false}
            validateOnBlur={false}
            validateOnMount={false}
            errors={errors}
            initialValues={
              {
                role: "either",
                name: "",
                api_url: "",
                dap_url: "",
                bearer_token: "",
              } as NewAggregator
            }
            onSubmit={handleSubmit}
          >
            {(props) => (
              <Form
                method="post"
                onSubmit={props.handleSubmit}
                noValidate
                autoComplete="off"
              >
                <RoleSelect {...props} />
                <Name {...props} />
                <ApiUrl {...props} />
                <DapUrl {...props} />
                <BearerToken {...props} />
                <Button
                  variant="primary"
                  type="submit"
                  disabled={navigation.state === "submitting"}
                >
                  Submit
                </Button>
              </Form>
            )}
          </Formik>
        </Col>
      </Row>
    </>
  );
}

function Name(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>Name</FormLabel>

      <FormControl
        type="text"
        name="name"
        autoComplete="off"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.name}
        isInvalid={!!props.errors.name}
        data-1p-ignore={true}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.name}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function BearerToken(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>Bearer Token</FormLabel>

      <FormControl
        type="text"
        name="bearer_token"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        autoComplete="off"
        value={props.values.bearer_token}
        isInvalid={!!props.errors.bearer_token}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.bearer_token}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function ApiUrl(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>API Url</FormLabel>

      <FormControl
        type="url"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.api_url}
        isInvalid={!!props.errors.api_url}
        autoComplete="off"
        name="api_url"
        placeholder="https://example.com"
        pattern="https://.*"
        required
      />
      <FormControl.Feedback type="invalid">
        {props.errors.api_url}
      </FormControl.Feedback>
    </FormGroup>
  );
}
function DapUrl(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>DAP Url</FormLabel>

      <FormControl
        type="url"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.dap_url}
        isInvalid={!!props.errors.dap_url}
        required
        autoComplete="off"
        placeholder="https://example.com"
        pattern="https://.*"
        name="dap_url"
      />
      <FormControl.Feedback type="invalid">
        {props.errors.dap_url}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function RoleSelect(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>Role</FormLabel>

      <FormSelect
        value={props.values.role}
        isInvalid={!!props.errors.role}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        name="role"
      >
        <option value="leader">Leader</option>
        <option value="helper">Helper</option>
        <option value="either">Either Leader or Helper</option>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.role}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <LinkContainer to="..">
        <Breadcrumb.Item>Aggregators</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>New</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}
