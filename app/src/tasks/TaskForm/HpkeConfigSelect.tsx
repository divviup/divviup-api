import { Await, useLoaderData } from "react-router-dom";
import FormControl from "react-bootstrap/FormControl";
import FormSelect from "react-bootstrap/FormSelect";
import React, { Suspense } from "react";
import { HpkeConfig } from "../../ApiClient";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";

export default function HpkeConfigSelect(props: Props) {
  const { hpkeConfigs } = useLoaderData() as {
    hpkeConfigs: Promise<HpkeConfig[]>;
  };
  const { setFieldValue } = props;

  React.useEffect(() => {
    hpkeConfigs.then((configs) => {
      if (configs.length === 1) setFieldValue("hpke_config_id", configs[0].id);
    });
  }, [hpkeConfigs, setFieldValue]);

  return (
    <TaskFormGroup controlId="hpke_config_id">
      <ShortHelpAndLabel
        fieldKey="hpke_config_id"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        isInvalid={!!props.errors.hpke_config_id}
        id="hpke-config-id"
        name="hpke_config_id"
      >
        <option disabled></option>
        <Suspense fallback={<option disabled>...</option>}>
          <Await resolve={hpkeConfigs}>
            {(hpkeConfigs: HpkeConfig[]) =>
              hpkeConfigs.map((hpkeConfig) => (
                <option key={hpkeConfig.id} value={hpkeConfig.id}>
                  {hpkeConfig.name}
                </option>
              ))
            }
          </Await>
        </Suspense>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.hpke_config_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
