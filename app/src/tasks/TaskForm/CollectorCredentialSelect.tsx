import { Await, useLoaderData } from "react-router-dom";
import FormControl from "react-bootstrap/FormControl";
import FormSelect from "react-bootstrap/FormSelect";
import React, { Suspense } from "react";
import { CollectorCredential } from "../../ApiClient";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export default function CollectorCredentialSelect(props: Props) {
  const { collectorCredentials } = useLoaderData() as {
    collectorCredentials: Promise<CollectorCredential[]>;
  };
  const { setFieldValue } = props;

  React.useEffect(() => {
    collectorCredentials.then((configs) => {
      if (configs.length === 1)
        setFieldValue("collector_credential_id", configs[0].id);
    });
  }, [collectorCredentials, setFieldValue]);

  return (
    <TaskFormGroup controlId="collector_credential_id">
      <ShortHelpAndLabel
        fieldKey="collector_credential_id"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        isInvalid={!!props.errors.collector_credential_id}
        id="collector-credential-id"
        name="collector_credential_id"
      >
        <option disabled></option>
        <Suspense fallback={<option disabled>...</option>}>
          <Await resolve={collectorCredentials}>
            {(collectorCredentials: CollectorCredential[]) =>
              collectorCredentials.map((collectorCredential) => (
                <option
                  key={collectorCredential.id}
                  value={collectorCredential.id}
                >
                  {collectorCredential.name}
                </option>
              ))
            }
          </Await>
        </Suspense>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.collector_credential_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
