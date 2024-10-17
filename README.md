# Josev binding

This repository hosts an [AFB binding](https://docs.redpesk.bzh/docs/en/master/developer-guides/afb-binding-overview.html) that implements the ISO-15118-2 part of a charging station by interfacing with [Josev](https://github.com/EcoG-io/josev).

Josev exposes its API through MQTT topics.

This binding exposes its API through verbs and events. It requires the use of the [MQTT extension](https://github.com/tux-evse/afb-mqtt-ext) to translate between AFB verbs / events and MQTT messages.

## Links to other bindings

This binding requires access to the following APIs:

- a charging binding, configured through the `charge_api` configuration key. It is used to access the current charge state and to command power delivery;
- an authentication binding, configured through the `auth_api` configuration key. It is used to requests authorization for EIM access;
- an electricity metering api, configured through the `meter_api` configuration key. It is used to monitor electricity consumption.

## EVSE Configuration

The charging station parameters and limits for Josev are stored as configuration of this binding. They will be sent to Josev when it starts.

Charging station parameters are stored in the `cs_parameters` configuration key and initial status and limits are stored in the `cs_status_and_limits` configuration key.

**Limitation**: only one EVSE with only one connector is supported by the binding for now.

One configuration example is provided [here](afb-binding/etc/binding-josev-ac-sample.json).

## MQTT extension configuration

The configuration file required by the MQTT extension so that bidirectionnal communication with a running instance of Josev ISO-15118-2 stack can take place is provided [here](afb-binding/etc/mqtt-config.yml).

## Invocation

Here is a pseudo command line for the invocation of the binding. Other required APIs are either imported through the use of `--ws-client` or loaded in the same security context (through `--config` and `--binding` options):

```
afb-binder \
  --config=... \
  --ws-client=... \
  --config=$CONFDIR/../../etc/binding-debug.json \
  --config=$CONFDIR/binding-josev-ac-sample.json \
  --binding=$LIBDIR/libafb_josev.so \
  --extension=$LIBDIR/libafb-mqtt-ext.so \
  --mqtt-config-file=$CONFDIR/mqtt-config.yml

```