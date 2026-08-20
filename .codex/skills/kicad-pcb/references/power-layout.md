# Power, thermal, and noisy-load layout

Use this branch for battery inputs, motors, heaters, solenoids, high-current
LEDs, converters, or other loads where generic signal rules are insufficient.

## Inputs to establish

- Nominal, minimum, transient, and reverse input voltage.
- Continuous and peak current per branch and total source limit.
- Copper weight, allowed temperature rise, allowed voltage drop, ambient and
  enclosure assumptions.
- Connector, fuse/protection, switch, regulator, inductor, diode, and capacitor
  current/thermal ratings.
- Motor/load noise, fault energy, cable length, and return-current path.

## Layout gates

- Size traces, pours, vias, pads, and connectors from the stated current,
  copper, temperature, and voltage-drop constraints; do not use a generic width
  table as proof.
- Keep high-di/dt loops compact. Place converter input bypassing at the switch
  loop and output network at the regulator/inductor path defined by its data.
- Give every load a deliberate supply and return path. Keep motor/load returns
  from sharing sensitive MCU, IMU, oscillator, or analog return impedance.
- Verify fuse and reverse/transient protection placement relative to the input.
- Check thermal copper, component spacing, airflow/enclosure limits, and heat
  transfer into temperature-sensitive parts.
- Record expected voltage drop and loss at continuous and peak current, plus
  the assumptions used.

Any unbounded current, unknown copper weight, underspecified fault protection,
or shared noisy return that could violate requirements blocks layout approval.
