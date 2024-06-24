/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

#[cfg(test)]
pub mod tests {
    pub fn fake_data() -> Vec<u8> {
        let data = Vec::from("hello world!");

        let repeat_times: usize = 1024 * 1024;

        data.iter()
            .cycle()
            .take(data.len() * repeat_times)
            .cloned()
            .collect()
    }
}
