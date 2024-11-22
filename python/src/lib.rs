use krec_rs::{
    ActuatorCommand, ActuatorConfig, ActuatorState, ImuQuaternion, ImuValues, KRec, Vec3,
};
use pyo3::exceptions::PyValueError;
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

    fn add_frame(&mut self, frame: &PyKRecFrame) {
        self.inner.add_frame(frame.inner.clone());
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

#[pyclass(name = "KRecFrame")]
#[derive(Debug, Clone)]
struct PyKRecFrame {
    inner: krec_rs::KRecFrame,
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