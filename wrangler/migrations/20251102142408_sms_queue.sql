DROP TABLE messages;
CREATE TABLE messages (
	id uuid primary key default gen_random_uuid(),
	content text NOT NULL ,
	phone text NOT NULL,
	processed bool NOT NULL default false,
	outgoing bool NOT NULL,
	inserted timestamp default LOCALTIMESTAMP(0),
	sent timestamp
);

CREATE OR REPLACE FUNCTION notifymsg() RETURNS trigger AS $$
DECLARE
BEGIN
  IF NEW.outgoing THEN
    PERFORM pg_notify('sent');
  ELSE 
    PERFORM pg_notify('received');
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER notify_msg AFTER INSERT ON messages FOR EACH ROW EXECUTE FUNCTION notifymsg();
