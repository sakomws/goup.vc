-- Create the requested GOUP city chapters once, without duplicating existing cities.
do $$
declare
    v_alliance_id uuid;
    v_category_id uuid;
begin
    select alliance_id into v_alliance_id from alliance where name = 'goup';
    if v_alliance_id is null then
        return;
    end if;

    insert into group_category (alliance_id, name)
    values (v_alliance_id, 'City Chapters')
    on conflict (alliance_id, name) do nothing;

    select group_category_id into v_category_id
    from group_category
    where alliance_id = v_alliance_id and name = 'City Chapters';

    with cities(region, city, slug) as (
        values
            ('North America','Atlanta','goup-atlanta'),('North America','Austin','goup-austin'),('North America','Boston','goup-boston'),('North America','Calgary','goup-calgary'),('North America','Chicago','goup-chicago'),('North America','Cincinnati','goup-cincinnati'),('North America','Dallas','goup-dallas'),('North America','Denver','goup-denver'),('North America','Detroit','goup-detroit'),('North America','Edmonton','goup-edmonton'),('North America','Houston','goup-houston'),('North America','Las Vegas','goup-las-vegas'),('North America','Los Angeles','goup-los-angeles'),('North America','Mexico City','goup-mexico-city'),('North America','Miami','goup-miami'),('North America','Minneapolis','goup-minneapolis'),('North America','Montréal','goup-montreal'),('North America','New York','goup-new-york'),('North America','Philadelphia','goup-philadelphia'),('North America','Phoenix','goup-phoenix'),('North America','Pittsburgh','goup-pittsburgh'),('North America','Portland','goup-portland'),('North America','Raleigh','goup-raleigh'),('North America','Sacramento','goup-sacramento'),('North America','Salt Lake City','goup-salt-lake-city'),('North America','San Diego','goup-san-diego'),('North America','San Francisco','goup-san-francisco'),('North America','Seattle','goup-seattle'),('North America','Toronto','goup-toronto'),('North America','Vancouver','goup-vancouver'),('North America','Washington, DC','goup-washington-dc'),('North America','Waterloo','goup-waterloo'),
            ('Asia & Pacific','Auckland','goup-auckland'),('Asia & Pacific','Bangkok','goup-bangkok'),('Asia & Pacific','Bengaluru','goup-bengaluru'),('Asia & Pacific','Brisbane','goup-brisbane'),('Asia & Pacific','Dubai','goup-dubai'),('Asia & Pacific','Ho Chi Minh City','goup-ho-chi-minh-city'),('Asia & Pacific','Hong Kong','goup-hong-kong'),('Asia & Pacific','Honolulu','goup-honolulu'),('Asia & Pacific','Jakarta','goup-jakarta'),('Asia & Pacific','Kuala Lumpur','goup-kuala-lumpur'),('Asia & Pacific','Manila','goup-manila'),('Asia & Pacific','Melbourne','goup-melbourne'),('Asia & Pacific','Mumbai','goup-mumbai'),('Asia & Pacific','New Delhi','goup-new-delhi'),('Asia & Pacific','Osaka','goup-osaka'),('Asia & Pacific','Seoul','goup-seoul'),('Asia & Pacific','Singapore','goup-singapore'),('Asia & Pacific','Sydney','goup-sydney'),('Asia & Pacific','Taipei','goup-taipei'),('Asia & Pacific','Tel Aviv-Yafo','goup-tel-aviv-yafo'),('Asia & Pacific','Tokyo','goup-tokyo'),
            ('South America','Bogotá','goup-bogota'),('South America','Buenos Aires','goup-buenos-aires'),('South America','Medellín','goup-medellin'),('South America','Rio de Janeiro','goup-rio-de-janeiro'),('South America','São Paulo','goup-sao-paulo'),
            ('Europe','Amsterdam','goup-amsterdam'),('Europe','Barcelona','goup-barcelona'),('Europe','Berlin','goup-berlin'),('Europe','Brussels','goup-brussels'),('Europe','Budapest','goup-budapest'),('Europe','Copenhagen','goup-copenhagen'),('Europe','Dublin','goup-dublin'),('Europe','Frankfurt','goup-frankfurt'),('Europe','Geneva','goup-geneva'),('Europe','Hamburg','goup-hamburg'),('Europe','Helsinki','goup-helsinki'),('Europe','Istanbul','goup-istanbul'),('Europe','Lausanne','goup-lausanne'),('Europe','Lisbon','goup-lisbon'),('Europe','London','goup-london'),('Europe','Madrid','goup-madrid'),('Europe','Milan','goup-milan'),('Europe','Munich','goup-munich'),('Europe','Oslo','goup-oslo'),('Europe','Paris','goup-paris'),('Europe','Prague','goup-prague'),('Europe','Rome','goup-rome'),('Europe','Stockholm','goup-stockholm'),('Europe','Vienna','goup-vienna'),('Europe','Warsaw','goup-warsaw'),('Europe','Zurich','goup-zurich'),
            ('Africa','Cape Town','goup-cape-town'),('Africa','Lagos','goup-lagos'),('Africa','Nairobi','goup-nairobi')
    )
    insert into "group" (alliance_id, group_category_id, name, slug, city, description_short, description, tags)
    select v_alliance_id, v_category_id, 'GOUP ' || city, slug, city,
           'GOUP city chapter for ' || city,
           'A GOUP city chapter for members, events, and local collaboration in ' || city || '.',
           array['City Chapter', region]
    from cities
    where not exists (
        select 1 from "group" g
        where g.alliance_id = v_alliance_id
          and g.deleted = false
          and (lower(g.name) = lower('GOUP ' || cities.city) or lower(g.city) = lower(cities.city))
    )
    on conflict (alliance_id, slug) do nothing;
end;
$$;
