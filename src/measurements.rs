use std::collections::VecDeque;

pub type Measurement = egui_plot::PlotPoint;

#[derive(Clone, Copy)]
pub struct SensorSample {
    t: u16,
    x: u16,
    y: u16,
    z: u16,
    timestamp: u64
}

impl SensorSample {
    pub fn empty() -> Self {
        Self {
            t: 0,
            x: 0,
            y: 0,
            z: 0,
            timestamp: 0
        }
    }

    pub fn new(t: u16, x: u16, y: u16, z: u16, timestamp: u64) -> Self {
        Self { t, x, y, z, timestamp }
    }
}

#[derive(PartialEq, Debug)]
pub struct SensorSampleMeasurement {
    t: Measurement,
    x: Measurement,
    y: Measurement,
    z: Measurement,
    timestamp: u64
}

impl SensorSampleMeasurement {
    pub fn from(s: SensorSample) -> Self {
        Self {
            t: Measurement::new(s.timestamp as f64, s.t),
            x: Measurement::new(s.timestamp as f64, s.x),
            y: Measurement::new(s.timestamp as f64, s.y),
            z: Measurement::new(s.timestamp as f64, s.z),
            timestamp: s.timestamp,
        }
    }
}

pub struct MeasurementWindow {
    pub values: VecDeque<SensorSampleMeasurement>,
    pub look_behind: usize,
}

impl MeasurementWindow {
    pub fn new_with_look_behind(look_behind: usize) -> Self {
        Self {
            values: VecDeque::new(),
            look_behind,
        }
    }

    pub fn add(&mut self, sensor_sample: SensorSample) {
        if let Some(last) = self.values.back() {
            if sensor_sample.timestamp < last.timestamp {
                self.values.clear()
            }
        }

        self.values.push_back(SensorSampleMeasurement::from(sensor_sample));

        match self.values.back().unwrap().timestamp.checked_sub(self.look_behind as u64) {
            Some(limit) => {
                while let Some(front) = self.values.front() {
                    if front.timestamp >= limit {
                        break;
                    }
                    self.values.pop_front();
                }
            },
            None => {
                if self.values.len() == 2 {
                    drop(self.values.pop_front())
                }
            }
        }
    }

    pub fn plot_values(&self) -> [egui_plot::PlotPoints; 4] {
        let t_plot = egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().map(|s| &s.t).copied()));
        let x_plot = egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().map(|s| &s.x).copied()));
        let y_plot = egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().map(|s| &s.y).copied()));
        let z_plot = egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().map(|s| &s.z).copied()));
        return [t_plot, x_plot, y_plot, z_plot];
    }
}

#[cfg(test)]
mod test {
    use egui_plot::PlotPoints;
    use super::*;

    #[test]
    fn empty_measurements() {
        let w = MeasurementWindow::new_with_look_behind(6000000);
        assert_eq!(w.values.len(), 0);
        assert_eq!(w.look_behind, 6000000);
    }

    #[test]
    fn appends_one_value() {
        let mut w = MeasurementWindow::new_with_look_behind(6000000);
        let sensor_sample = SensorSample::new(6500, 500, 500, 500, 5000000);
        w.add(sensor_sample);
        assert_eq!(
            w.values.into_iter().eq(vec![SensorSampleMeasurement::from(sensor_sample)]),
            true
        );
    }

    #[test]
    fn clears_on_out_of_order() {
        let mut w = MeasurementWindow::new_with_look_behind(6000000);
        let s1 = SensorSample::new(6500, 300, 300, 300, 3000000);
        let s2 = SensorSample::new(6500, 400, 400, 400, 4000000);
        let s3 = SensorSample::new(6500, 500, 500, 500, 5000000);
        let mut q = VecDeque::new();
        q.push_back(SensorSampleMeasurement::from(s3));
        w.add(s1);
        w.add(s2);
        w.add(s3);

        assert_eq!(
            w.values,
            q
        );
    }

    #[test]
    fn appends_several_values() {
        let mut w = MeasurementWindow::new_with_look_behind(1000000);
        let mut v: VecDeque<SensorSampleMeasurement> = VecDeque::new();

        for n in 1..=25 {
            let s = SensorSample::new(6500, (n * 10), (n * 10), (n * 10), ((n as u64) * 100000));
            w.add(s);
            v.push_back(SensorSampleMeasurement::from(s));
        }

        v.drain(..14);

        assert_eq!(
            w.values,
            v
        );
    }
}