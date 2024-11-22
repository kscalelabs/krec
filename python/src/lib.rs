use krec_rs::{
    ActuatorCommand, ActuatorConfig, ActuatorState, ImuQuaternion, ImuValues, KRec, Vec3,
};
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyIterator;
use tracing::{info, instrument};

/// A 3D vector with x, y, z components
#[pyclass(name = "Vec3")]
#[derive(Debug, Clone)]
struct PyVec3 {
    inner: Vec3,
}

#[pymethods]
impl PyVec3 {
    #[new]
    #[pyo3(text_signature = "(x=0.0, y=0.0, z=0.0, /, values=None)")]
    fn new(
        py: Python<'_>,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            // Try to convert from iterable
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut coords: Vec<f64> = Vec::new();
                for item in iter {
                    let value: f64 = item?.extract()?;
                    coords.push(value);
                }
                if coords.len() != 3 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 3 values for x, y, z",
                    ));
                }
                let mut inner = Vec3::default();
                inner.x = coords[0];
                inner.y = coords[1];
                inner.z = coords[2];
                return Ok(Self { inner });
            }
        }

        // Fall back to individual coordinates
        let mut inner = Vec3::default();
        inner.x = x.unwrap_or(0.0);
        inner.y = y.unwrap_or(0.0);
        inner.z = z.unwrap_or(0.0);
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "Vec3(x={}, y={}, z={})",
            self.inner.x, self.inner.y, self.inner.z
        )
    }

    #[getter]
    fn get_x(&self) -> f64 {
        self.inner.x
    }
    #[getter]
    fn get_y(&self) -> f64 {
        self.inner.y
    }
    #[getter]
    fn get_z(&self) -> f64 {
        self.inner.z
    }
}

/// A quaternion representing 3D rotation
#[pyclass(name = "IMUQuaternion")]
#[derive(Debug, Clone)]
struct PyIMUQuaternion {
    inner: ImuQuaternion,
}

#[pymethods]
impl PyIMUQuaternion {
    #[new]
    #[pyo3(text_signature = "(x=0.0, y=0.0, z=0.0, w=1.0, /, values=None)")]
    fn new(
        py: Python<'_>,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        w: Option<f64>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut coords: Vec<f64> = Vec::new();
                for item in iter {
                    let value: f64 = item?.extract()?;
                    coords.push(value);
                }
                if coords.len() != 4 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 4 values for x, y, z, w",
                    ));
                }
                let mut inner = ImuQuaternion::default();
                inner.x = coords[0];
                inner.y = coords[1];
                inner.z = coords[2];
                inner.w = coords[3];
                return Ok(Self { inner });
            }
        }

        let mut inner = ImuQuaternion::default();
        inner.x = x.unwrap_or(0.0);
        inner.y = y.unwrap_or(0.0);
        inner.z = z.unwrap_or(0.0);
        inner.w = w.unwrap_or(1.0); // w=1.0 represents no rotation
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "IMUQuaternion(x={}, y={}, z={}, w={})",
            self.inner.x, self.inner.y, self.inner.z, self.inner.w
        )
    }

    #[getter]
    fn get_x(&self) -> f64 {
        self.inner.x
    }
    #[getter]
    fn get_y(&self) -> f64 {
        self.inner.y
    }
    #[getter]
    fn get_z(&self) -> f64 {
        self.inner.z
    }
    #[getter]
    fn get_w(&self) -> f64 {
        self.inner.w
    }
}

/// IMU sensor values including acceleration, gyroscope, and orientation data
#[pyclass(name = "IMUValues")]
#[derive(Debug, Clone)]
struct PyIMUValues {
    inner: ImuValues,
}

#[pymethods]
impl PyIMUValues {
    #[new]
    #[pyo3(text_signature = "(accel=None, gyro=None, mag=None, quaternion=None, /, values=None)")]
    fn new(
        py: Python<'_>,
        accel: Option<PyVec3>,
        gyro: Option<PyVec3>,
        mag: Option<PyVec3>,
        quaternion: Option<PyIMUQuaternion>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut items = Vec::new();
                for item in iter {
                    let item = item?;
                    if !item.is_none() {
                        items.push(item.to_object(py));
                    } else {
                        items.push(py.None());
                    }
                }
                if items.len() != 4 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 4 values: [accel, gyro, mag, quaternion]",
                    ));
                }
                let mut inner = ImuValues::default();
                if !items[0].is_none(py) {
                    inner.accel = Some(items[0].extract::<PyVec3>(py)?.inner);
                }
                if !items[1].is_none(py) {
                    inner.gyro = Some(items[1].extract::<PyVec3>(py)?.inner);
                }
                if !items[2].is_none(py) {
                    inner.mag = Some(items[2].extract::<PyVec3>(py)?.inner);
                }
                if !items[3].is_none(py) {
                    inner.quaternion = Some(items[3].extract::<PyIMUQuaternion>(py)?.inner);
                }
                return Ok(Self { inner });
            }
        }

        let mut inner = ImuValues::default();
        if let Some(a) = accel {
            inner.accel = Some(a.inner);
        }
        if let Some(g) = gyro {
            inner.gyro = Some(g.inner);
        }
        if let Some(m) = mag {
            inner.mag = Some(m.inner);
        }
        if let Some(q) = quaternion {
            inner.quaternion = Some(q.inner);
        }
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        let accel = self
            .get_accel()
            .map(|v| v.__repr__())
            .unwrap_or_else(|| "None".to_string());
        let gyro = self
            .get_gyro()
            .map(|v| v.__repr__())
            .unwrap_or_else(|| "None".to_string());
        let quat = self
            .get_quaternion()
            .map(|q| q.__repr__())
            .unwrap_or_else(|| "None".to_string());
        format!(
            "IMUValues(accel={}, gyro={}, quaternion={})",
            accel, gyro, quat
        )
    }

    #[getter]
    fn get_accel(&self) -> Option<PyVec3> {
        self.inner.accel.clone().map(|v| PyVec3 { inner: v })
    }

    #[setter]
    fn set_accel(&mut self, value: Option<PyVec3>) {
        self.inner.accel = value.map(|v| v.inner);
    }

    #[getter]
    fn get_gyro(&self) -> Option<PyVec3> {
        self.inner.gyro.clone().map(|v| PyVec3 { inner: v })
    }

    #[setter]
    fn set_gyro(&mut self, value: Option<PyVec3>) {
        self.inner.gyro = value.map(|v| v.inner);
    }

    #[getter]
    fn get_quaternion(&self) -> Option<PyIMUQuaternion> {
        self.inner
            .quaternion
            .clone()
            .map(|q| PyIMUQuaternion { inner: q })
    }

    #[setter]
    fn set_quaternion(&mut self, value: Option<PyIMUQuaternion>) {
        self.inner.quaternion = value.map(|q| q.inner);
    }
}

/// State information for a single actuator
#[pyclass(name = "ActuatorState")]
#[derive(Debug, Clone)]
struct PyActuatorState {
    inner: ActuatorState,
}

#[pymethods]
impl PyActuatorState {
    #[new]
    #[pyo3(
        text_signature = "(actuator_id, online=False, position=None, velocity=None, torque=None, temperature=None, voltage=None, current=None, /, values=None)"
    )]
    fn new(
        py: Python<'_>,
        actuator_id: u32,
        online: Option<bool>,
        position: Option<f64>,
        velocity: Option<f64>,
        torque: Option<f64>,
        temperature: Option<f64>,
        voltage: Option<f32>,
        current: Option<f32>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut items = Vec::new();
                for item in iter {
                    let item = item?;
                    items.push(item.to_object(py));
                }
                if items.len() != 7 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 7 values: [online, position, velocity, torque, temperature, voltage, current]"
                    ));
                }
                let mut inner = ActuatorState::default();
                inner.actuator_id = actuator_id;
                inner.online = items[0].extract(py)?;
                inner.position = if items[1].is_none(py) {
                    None
                } else {
                    Some(items[1].extract(py)?)
                };
                inner.velocity = if items[2].is_none(py) {
                    None
                } else {
                    Some(items[2].extract(py)?)
                };
                inner.torque = if items[3].is_none(py) {
                    None
                } else {
                    Some(items[3].extract(py)?)
                };
                inner.temperature = if items[4].is_none(py) {
                    None
                } else {
                    Some(items[4].extract(py)?)
                };
                inner.voltage = if items[5].is_none(py) {
                    None
                } else {
                    Some(items[5].extract(py)?)
                };
                inner.current = if items[6].is_none(py) {
                    None
                } else {
                    Some(items[6].extract(py)?)
                };
                return Ok(Self { inner });
            }
        }

        let mut inner = ActuatorState::default();
        inner.actuator_id = actuator_id;
        inner.online = online.unwrap_or(false);
        inner.position = position;
        inner.velocity = velocity;
        inner.torque = torque;
        inner.temperature = temperature;
        inner.voltage = voltage;
        inner.current = current;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "ActuatorState(id={}, online={}, pos={:?}, vel={:?}, torque={:?}, temp={:?}, volt={:?}, curr={:?})",
            self.inner.actuator_id,
            self.inner.online,
            self.inner.position,
            self.inner.velocity,
            self.inner.torque,
            self.inner.temperature,
            self.inner.voltage,
            self.inner.current
        )
    }

    #[getter]
    fn get_actuator_id(&self) -> u32 {
        self.inner.actuator_id
    }

    #[getter]
    fn get_online(&self) -> bool {
        self.inner.online
    }
    #[setter]
    fn set_online(&mut self, value: bool) {
        self.inner.online = value;
    }

    #[getter]
    fn get_position(&self) -> Option<f64> {
        self.inner.position
    }
    #[setter]
    fn set_position(&mut self, value: Option<f64>) {
        self.inner.position = value;
    }

    #[getter]
    fn get_velocity(&self) -> Option<f64> {
        self.inner.velocity
    }
    #[setter]
    fn set_velocity(&mut self, value: Option<f64>) {
        self.inner.velocity = value;
    }

    #[getter]
    fn get_torque(&self) -> Option<f64> {
        self.inner.torque
    }
    #[setter]
    fn set_torque(&mut self, value: Option<f64>) {
        self.inner.torque = value;
    }

    #[getter]
    fn get_temperature(&self) -> Option<f64> {
        self.inner.temperature
    }
    #[setter]
    fn set_temperature(&mut self, value: Option<f64>) {
        self.inner.temperature = value;
    }

    #[getter]
    fn get_voltage(&self) -> Option<f32> {
        self.inner.voltage
    }
    #[setter]
    fn set_voltage(&mut self, value: Option<f32>) {
        self.inner.voltage = value;
    }

    #[getter]
    fn get_current(&self) -> Option<f32> {
        self.inner.current
    }
    #[setter]
    fn set_current(&mut self, value: Option<f32>) {
        self.inner.current = value;
    }
}

/// Configuration for an actuator
#[pyclass(name = "ActuatorConfig")]
#[derive(Debug, Clone)]
struct PyActuatorConfig {
    inner: ActuatorConfig,
}

#[pymethods]
impl PyActuatorConfig {
    #[new]
    #[pyo3(
        text_signature = "(actuator_id, kp=None, kd=None, ki=None, max_torque=None, name=None, /, values=None)"
    )]
    fn new(
        py: Python<'_>,
        actuator_id: u32,
        kp: Option<f64>,
        kd: Option<f64>,
        ki: Option<f64>,
        max_torque: Option<f64>,
        name: Option<String>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut items = Vec::new();
                for item in iter {
                    let item = item?;
                    items.push(item.to_object(py));
                }
                if items.len() != 5 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 5 values: [kp, kd, ki, max_torque, name]",
                    ));
                }
                let mut inner = ActuatorConfig::default();
                inner.actuator_id = actuator_id;
                inner.kp = if items[0].is_none(py) {
                    None
                } else {
                    Some(items[0].extract(py)?)
                };
                inner.kd = if items[1].is_none(py) {
                    None
                } else {
                    Some(items[1].extract(py)?)
                };
                inner.ki = if items[2].is_none(py) {
                    None
                } else {
                    Some(items[2].extract(py)?)
                };
                inner.max_torque = if items[3].is_none(py) {
                    None
                } else {
                    Some(items[3].extract(py)?)
                };
                inner.name = if items[4].is_none(py) {
                    None
                } else {
                    Some(items[4].extract(py)?)
                };
                return Ok(Self { inner });
            }
        }

        let mut inner = ActuatorConfig::default();
        inner.actuator_id = actuator_id;
        inner.kp = kp;
        inner.kd = kd;
        inner.ki = ki;
        inner.max_torque = max_torque;
        inner.name = name;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "ActuatorConfig(id={}, kp={:?}, kd={:?}, ki={:?}, max_torque={:?}, name={:?})",
            self.inner.actuator_id,
            self.inner.kp,
            self.inner.kd,
            self.inner.ki,
            self.inner.max_torque,
            self.inner.name
        )
    }

    #[getter]
    fn get_actuator_id(&self) -> u32 {
        self.inner.actuator_id
    }

    #[getter]
    fn get_kp(&self) -> Option<f64> {
        self.inner.kp
    }
    #[setter]
    fn set_kp(&mut self, value: Option<f64>) {
        self.inner.kp = value;
    }

    #[getter]
    fn get_kd(&self) -> Option<f64> {
        self.inner.kd
    }
    #[setter]
    fn set_kd(&mut self, value: Option<f64>) {
        self.inner.kd = value;
    }

    #[getter]
    fn get_ki(&self) -> Option<f64> {
        self.inner.ki
    }
    #[setter]
    fn set_ki(&mut self, value: Option<f64>) {
        self.inner.ki = value;
    }

    #[getter]
    fn get_max_torque(&self) -> Option<f64> {
        self.inner.max_torque
    }
    #[setter]
    fn set_max_torque(&mut self, value: Option<f64>) {
        self.inner.max_torque = value;
    }

    #[getter]
    fn get_name(&self) -> Option<String> {
        self.inner.name.clone()
    }
    #[setter]
    fn set_name(&mut self, value: Option<String>) {
        self.inner.name = value;
    }
}

/// Command for an actuator
#[pyclass(name = "ActuatorCommand")]
#[derive(Debug, Clone)]
struct PyActuatorCommand {
    inner: ActuatorCommand,
}

#[pymethods]
impl PyActuatorCommand {
    #[new]
    #[pyo3(
        text_signature = "(actuator_id, position=0.0, velocity=0.0, effort=0.0, /, values=None)"
    )]
    fn new(
        py: Python<'_>,
        actuator_id: u32,
        position: Option<f32>,
        velocity: Option<f32>,
        effort: Option<f32>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut coords: Vec<f32> = Vec::new();
                for item in iter {
                    let value: f32 = item?.extract()?;
                    coords.push(value);
                }
                if coords.len() != 3 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 3 values for position, velocity, effort",
                    ));
                }
                let mut inner = ActuatorCommand::default();
                inner.actuator_id = actuator_id;
                inner.position = coords[0];
                inner.velocity = coords[1];
                inner.effort = coords[2];
                return Ok(Self { inner });
            }
        }

        let mut inner = ActuatorCommand::default();
        inner.actuator_id = actuator_id;
        inner.position = position.unwrap_or(0.0);
        inner.velocity = velocity.unwrap_or(0.0);
        inner.effort = effort.unwrap_or(0.0);
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "ActuatorCommand(id={}, pos={}, vel={}, effort={})",
            self.inner.actuator_id, self.inner.position, self.inner.velocity, self.inner.effort
        )
    }

    #[getter]
    fn get_actuator_id(&self) -> u32 {
        self.inner.actuator_id
    }

    #[getter]
    fn get_position(&self) -> f32 {
        self.inner.position
    }
    #[setter]
    fn set_position(&mut self, value: f32) {
        self.inner.position = value;
    }

    #[getter]
    fn get_velocity(&self) -> f32 {
        self.inner.velocity
    }
    #[setter]
    fn set_velocity(&mut self, value: f32) {
        self.inner.velocity = value;
    }

    #[getter]
    fn get_effort(&self) -> f32 {
        self.inner.effort
    }
    #[setter]
    fn set_effort(&mut self, value: f32) {
        self.inner.effort = value;
    }
}

#[pyclass(name = "KRec")]
#[derive(Debug, Clone)]
struct PyKRec {
    inner: KRec,
}

#[pymethods]
impl PyKRec {
    #[new]
    #[instrument]
    fn new(header: &PyKRecHeader) -> PyResult<Self> {
        info!("Creating new Python KRec wrapper");
        let _ = krec_rs::init();

        Ok(Self {
            inner: KRec::new(header.inner.clone()),
        })
    }

    /// Get a specific frame by index
    fn get_frame(&self, index: usize) -> PyResult<PyKRecFrame> {
        if index >= self.inner.frames.len() {
            return Err(PyIndexError::new_err(format!(
                "Frame index {} out of range (0-{})",
                index,
                self.inner.frames.len() - 1
            )));
        }
        Ok(PyKRecFrame {
            inner: self.inner.frames[index].clone(),
        })
    }

    /// Get all frames
    fn get_frames(&self) -> Vec<PyKRecFrame> {
        self.inner
            .frames
            .iter()
            .map(|f| PyKRecFrame { inner: f.clone() })
            .collect()
    }

    /// Add a frame to the recording
    fn add_frame(&mut self, frame: &PyKRecFrame) {
        self.inner.frames.push(frame.inner.clone());
    }

    /// Remove a frame at the specified index
    fn remove_frame(&mut self, index: usize) -> PyResult<()> {
        if index >= self.inner.frames.len() {
            return Err(PyIndexError::new_err(format!(
                "Frame index {} out of range (0-{})",
                index,
                self.inner.frames.len() - 1
            )));
        }
        self.inner.frames.remove(index);
        Ok(())
    }

    /// Clear all frames
    fn clear_frames(&mut self) {
        self.inner.frames.clear();
    }

    /// Get the number of frames
    #[getter]
    fn frame_count(&self) -> usize {
        self.inner.frames.len()
    }

    /// Get a frame by index (Python [] operator)
    fn __getitem__(&self, index: isize) -> PyResult<PyKRecFrame> {
        let len = self.inner.frames.len() as isize;
        let normalized_index = if index < 0 { len + index } else { index };

        if normalized_index < 0 || normalized_index >= len {
            return Err(PyIndexError::new_err(format!(
                "Frame index {} out of range (0-{})",
                index,
                len - 1
            )));
        }

        self.get_frame(normalized_index as usize)
    }

    /// Get the length (Python len() function)
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.frames.len())
    }

    /// Iterator support for frames
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<FrameIterator>> {
        let iter = FrameIterator {
            frames: slf.inner.frames.clone(),
            index: 0,
        };
        Python::with_gil(|py| Py::new(py, iter))
    }

    fn __repr__(&self) -> String {
        format!(
            "KRec(frames={}, header={})",
            self.inner.frames.len(),
            format!(
                "KRecHeader(uuid='{}', task='{}', robot_platform='{}', robot_serial='{}', configs={})",
                self.inner.header.uuid,
                self.inner.header.task,
                self.inner.header.robot_platform,
                self.inner.header.robot_serial,
                self.inner.header.actuator_configs.len()
            )
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Returns a detailed string representation of the KRec contents
    fn display(&self) -> String {
        let mut output = String::new();

        // Header information
        output.push_str("KRec Recording\n");
        output.push_str("==============\n\n");

        // Basic info
        output.push_str(&format!("Task: {}\n", self.inner.header.task));
        output.push_str(&format!(
            "Robot Platform: {}\n",
            self.inner.header.robot_platform
        ));
        output.push_str(&format!(
            "Robot Serial: {}\n",
            self.inner.header.robot_serial
        ));
        output.push_str(&format!("UUID: {}\n", self.inner.header.uuid));
        output.push_str(&format!(
            "Start Timestamp: {}\n",
            self.inner.header.start_timestamp
        ));
        output.push_str(&format!(
            "End Timestamp: {}\n",
            self.inner.header.end_timestamp
        ));

        // Actuator configs
        output.push_str(&format!(
            "\nActuator Configs ({})\n",
            self.inner.header.actuator_configs.len()
        ));
        output.push_str("----------------\n");
        for config in &self.inner.header.actuator_configs {
            output.push_str(&format!("ID {}: ", config.actuator_id));
            if let Some(name) = &config.name {
                output.push_str(&format!("{} ", name));
            }
            output.push_str(&format!(
                "(kp={:?}, kd={:?}, ki={:?}, max_torque={:?})\n",
                config.kp, config.kd, config.ki, config.max_torque
            ));
        }

        // Frames summary
        output.push_str(&format!("\nFrames ({})\n", self.inner.frames.len()));
        output.push_str("------------\n");
        if !self.inner.frames.is_empty() {
            let first_frame = &self.inner.frames[0];
            let last_frame = &self.inner.frames[self.inner.frames.len() - 1];
            output.push_str(&format!(
                "Time range: {} to {}\n",
                first_frame.video_timestamp, last_frame.video_timestamp
            ));

            // Sample frame details (first frame)
            output.push_str("\nFirst frame details:\n");
            output.push_str(&format!("  Frame number: {}\n", first_frame.frame_number));
            output.push_str(&format!(
                "  Inference step: {}\n",
                first_frame.inference_step
            ));
            output.push_str(&format!(
                "  Actuator states: {}\n",
                first_frame.actuator_states.len()
            ));
            if first_frame.actuator_commands.is_some() {
                output.push_str("  Has actuator commands: yes\n");
            }
            if first_frame.imu_values.is_some() {
                output.push_str("  Has IMU values: yes\n");
            }
        }

        output
    }

    /// Returns a more detailed string representation of a specific frame
    fn display_frame(&self, frame_number: usize) -> PyResult<String> {
        if frame_number >= self.inner.frames.len() {
            return Err(PyValueError::new_err(format!(
                "Frame number {} out of range (0-{})",
                frame_number,
                self.inner.frames.len() - 1
            )));
        }

        let frame = &self.inner.frames[frame_number];
        let mut output = String::new();

        output.push_str(&format!("Frame {}\n", frame_number));
        output.push_str("=========\n\n");
        output.push_str(&format!("Video timestamp: {}\n", frame.video_timestamp));
        output.push_str(&format!("Frame number: {}\n", frame.frame_number));
        output.push_str(&format!("Inference step: {}\n", frame.inference_step));

        // Actuator states
        output.push_str(&format!(
            "\nActuator States ({})\n",
            frame.actuator_states.len()
        ));
        output.push_str("---------------\n");
        for state in &frame.actuator_states {
            output.push_str(&format!("ID {}: ", state.actuator_id));
            output.push_str(&format!(
                "online={}, pos={:?}, vel={:?}, torque={:?}, temp={:?}, volt={:?}, curr={:?}\n",
                state.online,
                state.position,
                state.velocity,
                state.torque,
                state.temperature,
                state.voltage,
                state.current
            ));
        }

        // Actuator commands
        if let Some(cmd) = &frame.actuator_commands {
            output.push_str("\nActuator Commands\n");
            output.push_str("----------------\n");
            output.push_str(&format!(
                "ID {}: pos={}, vel={}, effort={}\n",
                cmd.actuator_id, cmd.position, cmd.velocity, cmd.effort
            ));
        }

        // IMU values
        if let Some(imu) = &frame.imu_values {
            output.push_str("\nIMU Values\n");
            output.push_str("----------\n");
            if let Some(accel) = &imu.accel {
                output.push_str(&format!(
                    "Accel: x={}, y={}, z={}\n",
                    accel.x, accel.y, accel.z
                ));
            }
            if let Some(gyro) = &imu.gyro {
                output.push_str(&format!("Gyro: x={}, y={}, z={}\n", gyro.x, gyro.y, gyro.z));
            }
            if let Some(mag) = &imu.mag {
                output.push_str(&format!("Mag: x={}, y={}, z={}\n", mag.x, mag.y, mag.z));
            }
            if let Some(quat) = &imu.quaternion {
                output.push_str(&format!(
                    "Quaternion: x={}, y={}, z={}, w={}\n",
                    quat.x, quat.y, quat.z, quat.w
                ));
            }
        }

        Ok(output)
    }

    /// Returns the header
    #[getter]
    fn header(&self) -> PyKRecHeader {
        PyKRecHeader {
            inner: self.inner.header.clone(),
        }
    }

    fn save(&self, path: &str) -> PyResult<()> {
        self.inner
            .save(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        let krec = KRec::load(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(Self { inner: krec })
    }

    fn combine_with_video(
        &self,
        video_path: &str,
        output_path: &str,
        uuid: &str,
        task: &str,
    ) -> PyResult<()> {
        // First save the KRec to a temporary file
        let temp_path = format!("{}.tmp.krec", output_path);
        self.save(&temp_path)?;

        // Combine with video
        krec_rs::combine_with_video(video_path, &temp_path, output_path, uuid, task)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        // Clean up temporary file
        std::fs::remove_file(&temp_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        Ok(())
    }
}

#[pyclass(name = "KRecHeader")]
#[derive(Debug, Clone)]
struct PyKRecHeader {
    inner: krec_rs::KRecHeader,
}

#[pymethods]
impl PyKRecHeader {
    #[new]
    #[pyo3(
        text_signature = "(uuid=None, task=None, robot_platform=None, robot_serial=None, start_timestamp=None, end_timestamp=None, /, values=None)"
    )]
    fn new(
        py: Python<'_>,
        uuid: Option<String>,
        task: Option<String>,
        robot_platform: Option<String>,
        robot_serial: Option<String>,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut items = Vec::new();
                for item in iter {
                    let item = item?;
                    items.push(item.to_object(py));
                }
                if items.len() != 6 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 6 values: [uuid, task, robot_platform, robot_serial, start_timestamp, end_timestamp]"
                    ));
                }
                let mut inner = krec_rs::KRecHeader::default();
                inner.uuid = items[0].extract::<String>(py)?;
                inner.task = items[1].extract::<String>(py)?;
                inner.robot_platform = items[2].extract::<String>(py)?;
                inner.robot_serial = items[3].extract::<String>(py)?;
                inner.start_timestamp = items[4].extract::<u64>(py)?;
                inner.end_timestamp = items[5].extract::<u64>(py)?;
                return Ok(Self { inner });
            }
        }

        let mut inner = krec_rs::KRecHeader::default();
        inner.uuid = uuid.unwrap_or_default();
        inner.task = task.unwrap_or_default();
        inner.robot_platform = robot_platform.unwrap_or_default();
        inner.robot_serial = robot_serial.unwrap_or_default();
        inner.start_timestamp = start_timestamp.unwrap_or(0);
        inner.end_timestamp = end_timestamp.unwrap_or(0);
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "KRecHeader(uuid='{}', task='{}', robot_platform='{}', robot_serial='{}', start_timestamp={}, end_timestamp={})",
            self.inner.uuid,
            self.inner.task,
            self.inner.robot_platform,
            self.inner.robot_serial,
            self.inner.start_timestamp,
            self.inner.end_timestamp
        )
    }

    // Getters and setters for basic fields
    #[getter]
    fn get_uuid(&self) -> String {
        self.inner.uuid.clone()
    }
    #[setter]
    fn set_uuid(&mut self, value: String) {
        self.inner.uuid = value;
    }

    #[getter]
    fn get_task(&self) -> String {
        self.inner.task.clone()
    }
    #[setter]
    fn set_task(&mut self, value: String) {
        self.inner.task = value;
    }

    #[getter]
    fn get_robot_platform(&self) -> String {
        self.inner.robot_platform.clone()
    }
    #[setter]
    fn set_robot_platform(&mut self, value: String) {
        self.inner.robot_platform = value;
    }

    #[getter]
    fn get_robot_serial(&self) -> String {
        self.inner.robot_serial.clone()
    }
    #[setter]
    fn set_robot_serial(&mut self, value: String) {
        self.inner.robot_serial = value;
    }

    #[getter]
    fn get_start_timestamp(&self) -> u64 {
        self.inner.start_timestamp
    }
    #[setter]
    fn set_start_timestamp(&mut self, value: u64) {
        self.inner.start_timestamp = value;
    }

    #[getter]
    fn get_end_timestamp(&self) -> u64 {
        self.inner.end_timestamp
    }
    #[setter]
    fn set_end_timestamp(&mut self, value: u64) {
        self.inner.end_timestamp = value;
    }

    // Methods for actuator configs
    fn add_actuator_config(&mut self, config: &PyActuatorConfig) {
        self.inner.actuator_configs.push(config.inner.clone());
    }

    fn get_actuator_configs(&self, _py: Python<'_>) -> Vec<PyActuatorConfig> {
        self.inner
            .actuator_configs
            .iter()
            .map(|config| PyActuatorConfig {
                inner: config.clone(),
            })
            .collect()
    }

    fn clear_actuator_configs(&mut self) {
        self.inner.actuator_configs.clear();
    }
}

#[pyclass(name = "KRecFrame")]
#[derive(Debug, Clone)]
struct PyKRecFrame {
    inner: krec_rs::KRecFrame,
}

#[pymethods]
impl PyKRecFrame {
    #[new]
    #[pyo3(
        text_signature = "(video_timestamp=None, frame_number=None, inference_step=None, /, values=None)"
    )]
    fn new(
        py: Python<'_>,
        video_timestamp: Option<u64>,
        frame_number: Option<u64>,
        inference_step: Option<u64>,
        values: Option<&PyAny>,
    ) -> PyResult<Self> {
        if let Some(values) = values {
            if let Ok(iter) = PyIterator::from_object(py, values) {
                let mut items = Vec::new();
                for item in iter {
                    let item = item?;
                    items.push(item.to_object(py));
                }
                if items.len() != 3 {
                    return Err(PyValueError::new_err(
                        "Iterable must contain exactly 3 values: [video_timestamp, frame_number, inference_step]"
                    ));
                }
                let mut inner = krec_rs::KRecFrame::default();
                inner.video_timestamp = items[0].extract::<u64>(py)?;
                inner.frame_number = items[1].extract::<u64>(py)?;
                inner.inference_step = items[2].extract::<u64>(py)?;
                return Ok(Self { inner });
            }
        }

        let mut inner = krec_rs::KRecFrame::default();
        inner.video_timestamp = video_timestamp.unwrap_or(0);
        inner.frame_number = frame_number.unwrap_or(0);
        inner.inference_step = inference_step.unwrap_or(0);
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "KRecFrame(video_timestamp={}, frame_number={}, inference_step={}, states={}, has_commands={}, has_imu={})",
            self.inner.video_timestamp,
            self.inner.frame_number,
            self.inner.inference_step,
            self.inner.actuator_states.len(),
            self.inner.actuator_commands.is_some(),
            self.inner.imu_values.is_some()
        )
    }

    // Getters and setters for basic fields
    #[getter]
    fn get_video_timestamp(&self) -> u64 {
        self.inner.video_timestamp
    }
    #[setter]
    fn set_video_timestamp(&mut self, value: u64) {
        self.inner.video_timestamp = value;
    }

    #[getter]
    fn get_frame_number(&self) -> u64 {
        self.inner.frame_number
    }
    #[setter]
    fn set_frame_number(&mut self, value: u64) {
        self.inner.frame_number = value;
    }

    #[getter]
    fn get_inference_step(&self) -> u64 {
        self.inner.inference_step
    }
    #[setter]
    fn set_inference_step(&mut self, value: u64) {
        self.inner.inference_step = value;
    }

    // Methods for actuator states
    fn add_actuator_state(&mut self, state: &PyActuatorState) {
        self.inner.actuator_states.push(state.inner.clone());
    }

    fn get_actuator_states(&self, _py: Python<'_>) -> Vec<PyActuatorState> {
        self.inner
            .actuator_states
            .iter()
            .map(|state| PyActuatorState {
                inner: state.clone(),
            })
            .collect()
    }

    fn clear_actuator_states(&mut self) {
        self.inner.actuator_states.clear();
    }

    // Methods for actuator commands
    fn set_actuator_commands(&mut self, commands: Option<&PyActuatorCommand>) {
        self.inner.actuator_commands = commands.map(|cmd| cmd.inner.clone());
    }

    fn get_actuator_commands(&self, _py: Python<'_>) -> Option<PyActuatorCommand> {
        self.inner
            .actuator_commands
            .as_ref()
            .map(|cmd| PyActuatorCommand { inner: cmd.clone() })
    }

    fn clear_actuator_commands(&mut self) {
        self.inner.actuator_commands = None;
    }

    // Methods for IMU values
    fn set_imu_values(&mut self, imu: Option<&PyIMUValues>) {
        self.inner.imu_values = imu.map(|imu| imu.inner.clone());
    }

    fn get_imu_values(&self, _py: Python<'_>) -> Option<PyIMUValues> {
        self.inner
            .imu_values
            .as_ref()
            .map(|imu| PyIMUValues { inner: imu.clone() })
    }

    fn clear_imu_values(&mut self) {
        self.inner.imu_values = None;
    }

    // Utility methods
    fn has_actuator_commands(&self) -> bool {
        self.inner.actuator_commands.is_some()
    }

    fn has_imu_values(&self) -> bool {
        self.inner.imu_values.is_some()
    }

    fn actuator_state_count(&self) -> usize {
        self.inner.actuator_states.len()
    }
}

/// Iterator for frames
#[pyclass]
struct FrameIterator {
    frames: Vec<krec_rs::KRecFrame>,
    index: usize,
}

#[pymethods]
impl FrameIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyKRecFrame> {
        if slf.index < slf.frames.len() {
            let frame = PyKRecFrame {
                inner: slf.frames[slf.index].clone(),
            };
            slf.index += 1;
            Some(frame)
        } else {
            None
        }
    }
}

#[pymodule]
fn krec(_py: Python, m: &PyModule) -> PyResult<()> {
    let _ = krec_rs::init();
    m.add_class::<PyVec3>()?;
    m.add_class::<PyIMUQuaternion>()?;
    m.add_class::<PyIMUValues>()?;
    m.add_class::<PyActuatorState>()?;
    m.add_class::<PyActuatorConfig>()?;
    m.add_class::<PyActuatorCommand>()?;
    m.add_class::<PyKRecFrame>()?;
    m.add_class::<PyKRecHeader>()?;
    m.add_class::<PyKRec>()?;
    Ok(())
}
