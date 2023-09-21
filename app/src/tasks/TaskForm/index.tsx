import {
  useNavigate,
  useNavigation,
  useParams,
  NavigateFunction,
} from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import Button from "react-bootstrap/Button";
import Col from "react-bootstrap/Col";
import Form from "react-bootstrap/Form";
import FormGroup from "react-bootstrap/FormGroup";
import React from "react";
import Row from "react-bootstrap/Row";
import { ApiClientContext } from "../../ApiClientContext";
import { LinkContainer } from "react-router-bootstrap";
import ApiClient, { NewTask, formikErrors } from "../../ApiClient";
import { Formik, FormikHelpers, FormikProps } from "formik";
import { AccountBreadcrumbs } from "../../util";
import LeaderAggregator from "./LeaderAggregator";
import HelperAggregator from "./HelperAggregator";
import QueryType from "./QueryType";
import HpkeConfigSelect from "./HpkeConfigSelect";
import TaskName from "./TaskName";
import MinBatchSize from "./MinBatchSize";
import TimePrecisionSeconds from "./TimePrecisionSeconds";
import VdafType from "./VdafType";
import VdafDetails from "./VdafDetails";
import { LongHelpText } from "./HelpText";

async function submit(
  apiClient: ApiClient,
  accountId: string,
  newTask: NewTask,
  actions: FormikHelpers<NewTask>,
  navigate: NavigateFunction,
) {
  const task = await apiClient.createTask(accountId, newTask);

  if ("error" in task) {
    actions.setErrors(formikErrors(task.error));
  } else {
    return navigate(`/accounts/${accountId}/tasks/${task.id}`);
  }
}

export type Field<O extends object> = {
  [K in keyof O & string]: O[K] extends object ? `${K}.${Field<O[K]>}` : K;
}[keyof O & string];

export type Props = FormikProps<NewTask> & {
  setFocusedField(k: Field<NewTask>): void;
};

export function TaskFormGroup({
  children,
  controlId,
}: {
  children: React.ReactNode;
  controlId?: string;
}) {
  return (
    <FormGroup className="mb-5" controlId={controlId}>
      {children}
    </FormGroup>
  );
}

export default function TaskForm() {
  const { accountId } = useParams();
  const apiClient = React.useContext(ApiClientContext);
  const navigate = useNavigate();
  const handleSubmit = React.useCallback(
    (values: NewTask, actions: FormikHelpers<NewTask>) =>
      submit(apiClient, accountId as string, values, actions, navigate),
    [apiClient, accountId, navigate],
  );
  const navigation = useNavigation();

  const [focusedField, setFocusedField] = React.useState<Field<NewTask> | null>(
    null,
  );

  const focus = React.useCallback(
    (event: React.FocusEvent<HTMLFormElement>) => {
      setFocusedField(event.target.name as unknown as Field<NewTask>);
    },
    [setFocusedField, navigate],
  );

  return (
    <>
      <Breadcrumbs />
      <Row style={{ height: "calc(100vh - 100px)" }}>
        <Col sm="6" style={{ maxHeight: "100%", overflowY: "auto" }}>
          <Formik
            validateOnChange={false}
            validateOnBlur={false}
            validateOnMount={false}
            initialValues={
              {
                min_batch_size: 100,
                name: "",
                max_batch_size: null,
                time_precision_seconds: 3600,
                hpke_config_id: null,
              } as unknown as NewTask
            }
            onSubmit={handleSubmit}
          >
            {(formikProps) => {
              const props = { ...formikProps, setFocusedField };
              return (
                <Form
                  className="mb-5"
                  method="post"
                  onSubmit={formikProps.handleSubmit}
                  noValidate
                  autoComplete="off"
                  onFocus={focus}
                >
                  <TaskName {...props} />
                  <LeaderAggregator {...props} />
                  <HelperAggregator {...props} />
                  <VdafType {...props} />
                  <VdafDetails {...props} />
                  <QueryType {...props} />
                  <MinBatchSize {...props} />
                  <HpkeConfigSelect {...props} />
                  <TimePrecisionSeconds {...props} />
                  <TaskFormGroup>
                    <Button
                      variant="primary"
                      type="submit"
                      disabled={navigation.state === "submitting"}
                    >
                      Submit
                    </Button>
                  </TaskFormGroup>
                </Form>
              );
            }}
          </Formik>
        </Col>
        <Col sm={6}>
          {focusedField ? <LongHelpText field={focusedField} /> : null}
        </Col>
      </Row>
    </>
  );
}

function Breadcrumbs() {
  const { accountId } = useParams();

  return (
    <AccountBreadcrumbs>
      <LinkContainer to={`/accounts/${accountId}/tasks`}>
        <Breadcrumb.Item>Tasks</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>new</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}
