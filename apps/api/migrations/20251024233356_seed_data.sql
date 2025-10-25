-- Seed Data for Equipment Troubleshooting
-- This creates:
-- 1. Admin user for testing
-- 2. Complete decision tree from the original Flask application

-- ============================================
-- 1. CREATE ADMIN USER
-- ============================================
-- Email: admin@example.com
-- Password: "admin123" (hashed with Argon2)
-- NOTE: Change this password after first login!
INSERT INTO users (email, password_hash, role, is_active)
VALUES (
    'admin@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$BTONz7qKTsQS26oF4vwjDg$sj+0TMwEMlU1170DWTHan6kBJkCh2z6RKfG/NjFUYZI',
    'ADMIN',
    true
);

-- ============================================
-- 2. CREATE QUESTIONS
-- ============================================
-- Using semantic_id to match the original YAML structure

-- Start question
INSERT INTO questions (semantic_id, text, category) VALUES
('start', 'What are you having trouble with?', 'root'),
('brush_check', 'What''s the issue?', 'brush'),
('chemical_check', 'Chemical troubleshooting', 'chemical'),
('hp_check', 'High Pressure troubleshooting', 'high_pressure'),
('not_spinning1', 'Check controller - Verify brush isn''t forced off through controller', 'brush'),
('not_deploying', 'Check controller - Verify brush air isn''t forced off through controller', 'brush'),
('timing_pressure', 'Timing/Pressure Issues', 'brush'),
('forced_off', 'Verify with Maintenance that the brush isn''t forced off for a reason', 'brush'),
('set_auto', 'Set brush to AUTO - does brush function now that it''s set to AUTO?', 'brush'),
('on_auto', 'Check MCC - Is there power the VFD?', 'brush'),
('vfd_check', 'Does the VFD display show an error, or normal operation? (Normal operation consists of the display smoothly ramping up and down each time the brush turns on/off)', 'brush'),
('breaker_check', 'Is the circuit breaker tripped?', 'electrical'),
('siezed', 'Equipment may be seized', 'mechanical'),
('force_brush_on', 'Force brush on check', 'brush'),
('record_vfd_fault', 'Record VFD fault', 'electrical'),
('abnormal_operation', 'What type of abnormal operation are you observing?', 'general'),
('noise_diagnosis', 'When does the noise occur?', 'mechanical'),
('vibration_diagnosis', 'Is the vibration constant or does it change with speed?', 'mechanical'),
('temperature_diagnosis', 'Where is the excessive heat concentrated?', 'mechanical'),
('motor_heat', 'Can you hear the motor running?', 'electrical'),
('performance_diagnosis', 'How would you describe the performance issue?', 'mechanical');

-- ============================================
-- 3. CREATE ANSWERS
-- ============================================
-- Answers for 'start' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'start' AND q2.semantic_id = 'brush_check';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Chemical',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'start' AND q2.semantic_id = 'chemical_check';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'High Pressure',
    q2.id,
    2
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'start' AND q2.semantic_id = 'hp_check';

-- Answers for 'brush_check' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Not Spinning',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'brush_check' AND q2.semantic_id = 'not_spinning1';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Not Deploying',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'brush_check' AND q2.semantic_id = 'not_deploying';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Timing/Pressure Issues',
    q2.id,
    2
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'brush_check' AND q2.semantic_id = 'timing_pressure';

-- Answers for 'not_spinning1' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush is forced OFF',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'not_spinning1' AND q2.semantic_id = 'forced_off';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush is On AUTO',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'not_spinning1' AND q2.semantic_id = 'on_auto';

-- Answers for 'forced_off' question (one has conclusion)
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush NOT forced off by maintenance team',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'forced_off' AND q2.semantic_id = 'set_auto';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Forced off by maintenance',
    'Maintenance has the brush forced off pending repair. Verify that a request/work order exists on MaintainX app https://app.getmaintainx.com',
    1
FROM questions WHERE semantic_id = 'forced_off';

-- Answers for 'set_auto' question
INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Yes, brush works now',
    'Congrats!! You fixed it!',
    0
FROM questions WHERE semantic_id = 'set_auto';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush still not turning on for vehicles when set to AUTO',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'set_auto' AND q2.semantic_id = 'on_auto';

-- Answers for 'on_auto' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'VFD has power (LED screen display ON, indicator lights illuminated)',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'on_auto' AND q2.semantic_id = 'vfd_check';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'VFD does not have power (LED screen display OFF/no lights on VFD)',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'on_auto' AND q2.semantic_id = 'breaker_check';

-- Answers for 'vfd_check' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Normal operation',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'vfd_check' AND q2.semantic_id = 'siezed';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'VFD does nothing',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'vfd_check' AND q2.semantic_id = 'force_brush_on';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'VFD displaying fault',
    q2.id,
    2
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'vfd_check' AND q2.semantic_id = 'record_vfd_fault';

-- Answers for 'breaker_check' question (both have conclusions)
INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Yes',
    'Reset circuit breaker. If it trips again immediately/trips as soon as the piece of equipment turns on, there may be a short circuit - create MaintainX request and contact maintenance/regional manager.',
    0
FROM questions WHERE semantic_id = 'breaker_check';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'No',
    'Likely loss of power to breaker, underlying electrical issue. Create MaintainX request and contact maintenance/regional manager.',
    1
FROM questions WHERE semantic_id = 'breaker_check';

-- Answers for 'not_deploying' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush air is forced OFF',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'not_deploying' AND q2.semantic_id = 'forced_off';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Brush air is On AUTO',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'not_deploying' AND q2.semantic_id = 'on_auto';

-- Answers for 'abnormal_operation' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Unusual noise (grinding, squealing, knocking)',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'abnormal_operation' AND q2.semantic_id = 'noise_diagnosis';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Excessive vibration',
    q2.id,
    1
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'abnormal_operation' AND q2.semantic_id = 'vibration_diagnosis';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Overheating',
    q2.id,
    2
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'abnormal_operation' AND q2.semantic_id = 'temperature_diagnosis';

INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Performance issues (slow, weak output)',
    q2.id,
    3
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'abnormal_operation' AND q2.semantic_id = 'performance_diagnosis';

-- Answers for 'noise_diagnosis' question (all have conclusions)
INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'During startup only',
    'Startup noise may indicate bearing wear or lubrication issues. Schedule maintenance to inspect bearings and apply appropriate lubrication.',
    0
FROM questions WHERE semantic_id = 'noise_diagnosis';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Continuously during operation',
    'Continuous noise indicates worn bearings, misalignment, or loose components. Stop equipment and inspect for: worn bearings, loose mounting bolts, misaligned drive components.',
    1
FROM questions WHERE semantic_id = 'noise_diagnosis';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Intermittently',
    'Intermittent noise may indicate loose parts or debris. Inspect for loose fasteners, foreign objects, or worn couplings.',
    2
FROM questions WHERE semantic_id = 'noise_diagnosis';

-- Answers for 'vibration_diagnosis' question (all have conclusions)
INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Constant at all speeds',
    'Constant vibration indicates imbalance or misalignment. Check: mounting bolts are tight, equipment is level, rotating components are balanced, shaft alignment.',
    0
FROM questions WHERE semantic_id = 'vibration_diagnosis';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Changes with speed',
    'Speed-dependent vibration suggests resonance or worn components. Check: bearing condition, belt tension (if applicable), foundation stability.',
    1
FROM questions WHERE semantic_id = 'vibration_diagnosis';

-- Answers for 'temperature_diagnosis' question
INSERT INTO answers (question_id, label, next_question_id, order_index)
SELECT
    q1.id,
    'Motor/drive unit',
    q2.id,
    0
FROM questions q1
CROSS JOIN questions q2
WHERE q1.semantic_id = 'temperature_diagnosis' AND q2.semantic_id = 'motor_heat';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Bearing areas',
    'Hot bearings indicate inadequate lubrication or bearing failure. Stop equipment immediately. Check lubrication level and condition. Bearings may need replacement.',
    1
FROM questions WHERE semantic_id = 'temperature_diagnosis';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Overall equipment temperature high',
    'Overall high temperature may indicate: blocked ventilation, ambient temperature too high, overloading, or coolant issues. Check cooling system and reduce load if possible.',
    2
FROM questions WHERE semantic_id = 'temperature_diagnosis';

-- Answers for 'motor_heat' question (all have conclusions)
INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Yes, motor is running',
    'Hot motor while running indicates: overloading, poor ventilation, or motor issues. Check amp draw against nameplate, clear any obstructions from cooling vents, reduce load if possible.',
    0
FROM questions WHERE semantic_id = 'motor_heat';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'No, motor is humming but not rotating',
    'Motor humming but not rotating indicates: locked rotor, failed start capacitor (single phase), or mechanical jam. TURN OFF IMMEDIATELY to prevent motor damage. Check for mechanical obstructions.',
    1
FROM questions WHERE semantic_id = 'motor_heat';

-- Answers for 'performance_diagnosis' question (all have conclusions)
INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Slower than normal operation',
    'Reduced speed may indicate: worn drive belts, low voltage, motor issues, or increased mechanical resistance. Check belt condition and tension, verify supply voltage, inspect for binding or drag.',
    0
FROM questions WHERE semantic_id = 'performance_diagnosis';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Reduced output/capacity',
    'Reduced output may indicate: worn components, clogged filters/screens, improper adjustments, or seal leaks. Check filters, inspect wear parts, verify all adjustments are per specification.',
    1
FROM questions WHERE semantic_id = 'performance_diagnosis';

INSERT INTO answers (question_id, label, conclusion_text, order_index)
SELECT
    id,
    'Intermittent operation',
    'Intermittent operation suggests: loose electrical connections, thermal overload cycling, or control system issues. Check all electrical connections, verify overload settings, test control components.',
    2
FROM questions WHERE semantic_id = 'performance_diagnosis';

-- ============================================
-- NOTE: Admin password placeholder
-- ============================================
-- The admin user password is set to a placeholder.
-- After running this migration, generate a real password hash:
--
-- cargo run --bin hash_password YOUR_PASSWORD
--
-- Then update the user:
-- UPDATE users SET password_hash = 'YOUR_GENERATED_HASH' WHERE email = 'admin@example.com';
