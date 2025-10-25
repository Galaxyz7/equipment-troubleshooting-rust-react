SELECT 
    (SELECT COUNT(*) FROM users) as user_count,
    (SELECT COUNT(*) FROM questions) as question_count,
    (SELECT COUNT(*) FROM answers) as answer_count;
