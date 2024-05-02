describe('stylex-keyframes test', ()=>{
    test('converts keyframes to CSS', ()=>{
        expect(styleXKeyframes({
            from: {
                backgroundColor: 'red'
            },
            to: {
                backgroundColor: 'blue'
            }
        })).toMatchInlineSnapshot(`
      [
        "xbopttm-B",
        {
          "ltr": "@keyframes xbopttm-B{from{background-color:red;}to{background-color:blue;}}",
          "priority": 1,
          "rtl": null,
        },
      ]
    `);
    });
    test('generates RTL-specific keyframes', ()=>{
        expect(styleXKeyframes({
            from: {
                start: 0
            },
            to: {
                start: 500
            }
        })).toMatchInlineSnapshot(`
      [
        "x1id2van-B",
        {
          "ltr": "@keyframes x1id2van-B{from{inset-inline-start:0px;}to{inset-inline-start:500px;}}",
          "priority": 1,
          "rtl": null,
        },
      ]
    `);
    });
});
